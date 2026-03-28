use std::collections::HashMap;

use md5_digest::{Digest, Md5};
use ttf_parser::Face;

// ====== glyf 表 flag 位定义 ======
const ON_CURVE_POINT: u8 = 0x01;
const X_SHORT_VECTOR: u8 = 0x02;
const Y_SHORT_VECTOR: u8 = 0x04;
const REPEAT_FLAG: u8 = 0x08;
const X_IS_SAME_OR_POSITIVE: u8 = 0x10;
const Y_IS_SAME_OR_POSITIVE: u8 = 0x20;

/// 从 glyf 表原始字节中解析单个简单字形的坐标、标志和轮廓端点。
///
/// 返回 `(coordinates, flags, end_pts_of_contours)` ——
/// 精确复刻 Python `fontTools` 中 `Glyph` 对象暴露的三个核心属性。
///
/// # 参数
/// - `data`: glyf 表中该字形的原始字节切片
/// - `num_contours`: 轮廓数（必须 > 0，即简单字形）
fn parse_simple_glyph(
    data: &[u8],
    num_contours: usize,
) -> Option<(Vec<(i16, i16)>, Vec<u8>, Vec<u16>)> {
    // 头部: numberOfContours(2) + xMin(2) + yMin(2) + xMax(2) + yMax(2) = 10 字节
    if data.len() < 10 {
        return None;
    }

    let mut offset = 10; // 跳过头部（numberOfContours + bbox 已在调用方读取）

    // 读取 endPtsOfContours
    if data.len() < offset + num_contours * 2 {
        return None;
    }
    let mut end_pts = Vec::with_capacity(num_contours);
    for _ in 0..num_contours {
        let ep = u16::from_be_bytes([data[offset], data[offset + 1]]);
        end_pts.push(ep);
        offset += 2;
    }

    // 总点数 = max(endPtsOfContours) + 1
    let num_points = match end_pts.last() {
        Some(&last) => last as usize + 1,
        None => return None,
    };

    // 跳过 instructions
    if data.len() < offset + 2 {
        return None;
    }
    let instruction_length =
        u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
    offset += 2;
    if data.len() < offset + instruction_length {
        return None;
    }
    offset += instruction_length;

    // 解析 flags（支持 REPEAT_FLAG 压缩）
    let mut flags = Vec::with_capacity(num_points);
    while flags.len() < num_points {
        if offset >= data.len() {
            return None;
        }
        let flag = data[offset];
        offset += 1;
        flags.push(flag);

        if flag & REPEAT_FLAG != 0 {
            if offset >= data.len() {
                return None;
            }
            let repeat_count = data[offset] as usize;
            offset += 1;
            for _ in 0..repeat_count {
                if flags.len() >= num_points {
                    break;
                }
                flags.push(flag);
            }
        }
    }

    // 解析 X 坐标（差分编码）
    let mut x_coords = Vec::with_capacity(num_points);
    let mut x: i16 = 0;
    for &flag in &flags[..num_points] {
        if flag & X_SHORT_VECTOR != 0 {
            // 1 字节
            if offset >= data.len() {
                return None;
            }
            let dx = data[offset] as i16;
            offset += 1;
            if flag & X_IS_SAME_OR_POSITIVE != 0 {
                x += dx; // 正方向
            } else {
                x -= dx; // 负方向
            }
        } else if flag & X_IS_SAME_OR_POSITIVE != 0 {
            // 0 字节差分（与上一个相同）
        } else {
            // 2 字节有符号差分
            if offset + 1 >= data.len() {
                return None;
            }
            let dx = i16::from_be_bytes([data[offset], data[offset + 1]]);
            offset += 2;
            x += dx;
        }
        x_coords.push(x);
    }

    // 解析 Y 坐标（差分编码）
    let mut y_coords = Vec::with_capacity(num_points);
    let mut y: i16 = 0;
    for &flag in &flags[..num_points] {
        if flag & Y_SHORT_VECTOR != 0 {
            if offset >= data.len() {
                return None;
            }
            let dy = data[offset] as i16;
            offset += 1;
            if flag & Y_IS_SAME_OR_POSITIVE != 0 {
                y += dy;
            } else {
                y -= dy;
            }
        } else if flag & Y_IS_SAME_OR_POSITIVE != 0 {
            // 0 字节差分
        } else {
            if offset + 1 >= data.len() {
                return None;
            }
            let dy = i16::from_be_bytes([data[offset], data[offset + 1]]);
            offset += 2;
            y += dy;
        }
        y_coords.push(y);
    }

    // 组合坐标
    let coordinates: Vec<(i16, i16)> = x_coords
        .into_iter()
        .zip(y_coords)
        .collect();

    Some((coordinates, flags, end_pts))
}

/// 计算字形的 MD5 哈希。
///
/// 精确复刻 Python `cxsecret_font.py` 中的 `hash_glyph()` 函数：
/// 1. 遍历每个轮廓的每个坐标点（按 `endPtsOfContours` 确定边界）
/// 2. 对每个点提取 `x`、`y` 坐标（有符号整数的十进制表示）和 `flag & 0x01`
/// 3. 拼接为 `"{x}{y}{flag}"` 字符串
/// 4. 对完整拼接结果做 MD5
/// 5. `numberOfContours <= 0` 时返回空字符串（复合字形跳过）
fn hash_glyph(glyph_data: &[u8]) -> String {
    if glyph_data.len() < 2 {
        return String::new();
    }

    // 读取 numberOfContours（i16）
    let num_contours =
        i16::from_be_bytes([glyph_data[0], glyph_data[1]]);

    // 复合字形或空字形跳过
    if num_contours <= 0 {
        return String::new();
    }

    let parsed = match parse_simple_glyph(glyph_data, num_contours as usize)
    {
        Some(v) => v,
        None => return String::new(),
    };
    let (coordinates, flags, end_pts) = parsed;

    // 精确复刻 Python 的拼接逻辑
    let mut pos_data = String::new();
    let mut last_index: usize = 0;

    for i in 0..end_pts.len() {
        let end_point = end_pts[i] as usize;
        for j in last_index..=end_point {
            if j >= coordinates.len() {
                break;
            }
            let (x, y) = coordinates[j];
            let flag = flags[j] & ON_CURVE_POINT;
            // Python: f"{x}{y}{flag}" —— 有符号整数的十进制表示
            pos_data.push_str(&x.to_string());
            pos_data.push_str(&y.to_string());
            pos_data.push_str(&flag.to_string());
        }
        last_index = end_point + 1;
    }

    // MD5 哈希
    let mut hasher = Md5::new();
    hasher.update(pos_data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// 获取 TrueType 字体表的原始字节。
///
/// 使用 `ttf-parser` 的 `RawFace` API 获取指定表的数据切片。
fn get_table_data<'a>(face: &'a Face<'a>, tag: &[u8; 4]) -> Option<&'a [u8]> {
    face.raw_face().table(ttf_parser::Tag::from_bytes(tag))
}

/// 从 loca 表获取字形在 glyf 表中的偏移范围。
///
/// TrueType loca 表有两种格式：
/// - 短格式（`indexToLocFormat == 0`）：每个条目 2 字节，值 * 2 = 实际偏移
/// - 长格式（`indexToLocFormat == 1`）：每个条目 4 字节，值 = 实际偏移
fn get_glyph_offset_range(
    head_data: &[u8],
    loca_data: &[u8],
    glyph_id: u16,
) -> Option<(usize, usize)> {
    // head 表偏移 50 处是 indexToLocFormat（i16）
    if head_data.len() < 52 {
        return None;
    }
    let index_to_loc_format =
        i16::from_be_bytes([head_data[50], head_data[51]]);

    let (start, end) = if index_to_loc_format == 0 {
        // 短格式：每个条目 2 字节
        let idx = glyph_id as usize;
        if loca_data.len() < (idx + 2) * 2 {
            return None;
        }
        let s = u16::from_be_bytes([
            loca_data[idx * 2],
            loca_data[idx * 2 + 1],
        ]) as usize
            * 2;
        let e = u16::from_be_bytes([
            loca_data[(idx + 1) * 2],
            loca_data[(idx + 1) * 2 + 1],
        ]) as usize
            * 2;
        (s, e)
    } else {
        // 长格式：每个条目 4 字节
        let idx = glyph_id as usize;
        if loca_data.len() < (idx + 2) * 4 {
            return None;
        }
        let s = u32::from_be_bytes([
            loca_data[idx * 4],
            loca_data[idx * 4 + 1],
            loca_data[idx * 4 + 2],
            loca_data[idx * 4 + 3],
        ]) as usize;
        let e = u32::from_be_bytes([
            loca_data[(idx + 1) * 4],
            loca_data[(idx + 1) * 4 + 1],
            loca_data[(idx + 1) * 4 + 2],
            loca_data[(idx + 1) * 4 + 3],
        ]) as usize;
        (s, e)
    };

    if start == end {
        return None; // 空字形
    }

    Some((start, end))
}

/// 遍历 cmap 表获取所有 Unicode → GlyphId 映射。
///
/// 对应 Python `TTFont` 中通过 `font_file["glyf"].glyphOrder` + `table.glyphs[name]`
/// 获取字形的流程。在 Rust 中通过 cmap 表直接获取 unicode 码点到字形 ID 的映射。
fn get_unicode_mappings(face: &Face) -> Vec<(u32, u16)> {
    let mut mappings = Vec::new();

    if let Some(cmap) = face.tables().cmap {
        for subtable in cmap.subtables {
            if !subtable.is_unicode() {
                continue;
            }
            subtable.codepoints(|cp| {
                if let Some(gid) = subtable.glyph_index(cp) {
                    mappings.push((cp, gid.0));
                }
            });
        }
    }

    // 去重（多个 cmap 子表可能有重复映射）
    mappings.sort_unstable();
    mappings.dedup();

    mappings
}

/// 从字体二进制数据生成字形哈希映射表。
///
/// 精确对应 Python `cxsecret_font.py` 中的 `font2map()` 函数。
///
/// 返回 `HashMap<String, String>`，key 为 `"uniXXXX"` 格式的字形名称，
/// value 为该字形的 MD5 哈希值。
///
/// # 参数
/// - `font_data`: TTF 字体的原始二进制数据
pub fn font2map(font_data: &[u8]) -> HashMap<String, String> {
    let mut result = HashMap::new();

    let face = match Face::parse(font_data, 0) {
        Ok(f) => f,
        Err(_) => return result,
    };

    // 获取所需的原始表数据
    let glyf_data = match get_table_data(&face, b"glyf") {
        Some(d) => d,
        None => return result,
    };
    let loca_data = match get_table_data(&face, b"loca") {
        Some(d) => d,
        None => return result,
    };
    let head_data = match get_table_data(&face, b"head") {
        Some(d) => d,
        None => return result,
    };

    // 遍历 cmap 获取 unicode 映射
    let mappings = get_unicode_mappings(&face);

    for (codepoint, glyph_id) in mappings {
        // 构造 "uniXXXX" 名称，与 Python fontTools 的命名一致
        let glyph_name = format!("uni{:04X}", codepoint);

        // 只处理 "uni" 开头的字形（与 Python 一致）
        // 这里所有 cmap 来源的都是 "uni" 开头的

        // 从 loca 表获取字形在 glyf 表中的位置
        let range = match get_glyph_offset_range(head_data, loca_data, glyph_id)
        {
            Some(r) => r,
            None => continue,
        };

        let (start, end) = range;
        if start >= glyf_data.len() || end > glyf_data.len() || start >= end {
            continue;
        }

        let glyph_bytes = &glyf_data[start..end];
        let hash = hash_glyph(glyph_bytes);
        if !hash.is_empty() {
            result.insert(glyph_name, hash);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_glyph_empty_data() {
        assert_eq!(hash_glyph(&[]), "");
        assert_eq!(hash_glyph(&[0x00]), "");
    }

    #[test]
    fn test_hash_glyph_compound_glyph() {
        // numberOfContours = -1（复合字形）→ 返回空串
        let data = (-1_i16).to_be_bytes();
        assert_eq!(hash_glyph(&data), "");
    }

    #[test]
    fn test_hash_glyph_zero_contours() {
        // numberOfContours = 0 → 返回空串
        let data = 0_i16.to_be_bytes();
        assert_eq!(hash_glyph(&data), "");
    }

    #[test]
    fn test_parse_simple_glyph_minimal() {
        // 构造一个最小的单轮廓字形
        // numberOfContours = 1, bbox = 0,0,100,100
        // endPtsOfContours = [2] (3 个点)
        // instructionLength = 0
        // flags: [0x01, 0x01, 0x01] (all on-curve, 2-byte coords, no repeat)
        // x deltas: 0, 100, 0 (2-byte signed)
        // y deltas: 0, 0, 100 (2-byte signed)

        let mut data: Vec<u8> = Vec::new();
        // numberOfContours
        data.extend_from_slice(&1_i16.to_be_bytes());
        // bbox: xMin, yMin, xMax, yMax
        data.extend_from_slice(&0_i16.to_be_bytes());
        data.extend_from_slice(&0_i16.to_be_bytes());
        data.extend_from_slice(&100_i16.to_be_bytes());
        data.extend_from_slice(&100_i16.to_be_bytes());
        // endPtsOfContours[0] = 2
        data.extend_from_slice(&2_u16.to_be_bytes());
        // instructionLength = 0
        data.extend_from_slice(&0_u16.to_be_bytes());
        // flags: 3 个点，都是 on-curve，2 字节坐标
        // flag = 0x01 (ON_CURVE_POINT), no X_SHORT, no Y_SHORT, no REPEAT
        //   X_IS_SAME = 0 → 读 2 字节
        //   Y_IS_SAME = 0 → 读 2 字节
        data.push(0x01);
        data.push(0x01);
        data.push(0x01);
        // X deltas: 0, 100, -100 (累积: 0, 100, 0)
        data.extend_from_slice(&0_i16.to_be_bytes());
        data.extend_from_slice(&100_i16.to_be_bytes());
        data.extend_from_slice(&(-100_i16).to_be_bytes());
        // Y deltas: 0, 0, 100 (累积: 0, 0, 100)
        data.extend_from_slice(&0_i16.to_be_bytes());
        data.extend_from_slice(&0_i16.to_be_bytes());
        data.extend_from_slice(&100_i16.to_be_bytes());

        let hash = hash_glyph(&data);
        assert!(!hash.is_empty(), "简单字形应产生非空哈希");
        assert_eq!(hash.len(), 32, "MD5 哈希应为 32 个十六进制字符");

        // 手动计算预期值:
        // 坐标: (0,0), (100,0), (0,100)
        // flag & 0x01: 1, 1, 1
        // 拼接: "001" + "10001" + "01001"
        // = "00110001001001" — 不对，让我重新算
        // 点0: x=0, y=0, flag=1 → "001"
        // 点1: x=100, y=0, flag=1 → "10001"
        // 点2: x=0, y=100, flag=1 → "01001"
        // 拼接: "0011000101001"
        let expected_input = "0011000101001";
        let mut hasher = md5::Md5::new();
        hasher.update(expected_input.as_bytes());
        let expected_hash = format!("{:x}", hasher.finalize());
        assert_eq!(hash, expected_hash);
    }

    #[test]
    fn test_font2map_invalid_data() {
        // 无效字体数据应返回空映射
        let result = font2map(&[0, 1, 2, 3]);
        assert!(result.is_empty());
    }
}
