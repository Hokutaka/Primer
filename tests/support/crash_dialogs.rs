// 異常終了を検証する子プロセスがWindowsの対話ダイアログで待たないようにします。
pub fn suppress() {
    #[cfg(windows)]
    {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn SetErrorMode(mode: u32) -> u32;
            }
            // 子プロセスにも継承されます。テスト専用プロセス内だけの設定です。
            unsafe {
                SetErrorMode(0x0001 | 0x0002);
            }
        });
    }
}
