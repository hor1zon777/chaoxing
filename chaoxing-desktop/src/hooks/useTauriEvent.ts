import { useRef, useCallback } from "react";
import { Channel } from "@tauri-apps/api/core";

/**
 * 自定义 hook：创建 Tauri Channel 并绑定消息回调。
 *
 * 返回一个 getChannel 函数，调用后获取 Channel 实例。
 * Channel 会在首次调用 getChannel 时惰性创建，
 * 回调函数始终指向最新的 onMessage，无需担心闭包过期。
 */
export function useTauriChannel<T>(onMessage: (event: T) => void) {
  const channelRef = useRef<Channel<T> | null>(null);
  const callbackRef = useRef(onMessage);
  callbackRef.current = onMessage;

  const getChannel = useCallback(() => {
    if (!channelRef.current) {
      channelRef.current = new Channel<T>();
      channelRef.current.onmessage = (event: T) => {
        callbackRef.current(event);
      };
    }
    return channelRef.current;
  }, []);

  return getChannel;
}
