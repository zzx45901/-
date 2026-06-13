use std::fs::OpenOptions;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    print!("请输入 PAK 文件路径: ");
    io::stdout().flush()?;
    let mut path = String::new();
    io::stdin().read_line(&mut path)?;
    let path = path.trim();

    let target_mb = 500u64;
    let target_size = target_mb * 1024 * 1024;

    let file = OpenOptions::new().read(true).write(true).open(path)?;
    let current_size = file.metadata()?.len();
    if current_size < target_size {
        println!("当前大小 {} 字节 → 扩展至 {} 字节...", current_size, target_size);
        file.set_len(target_size)?;
        println!("完成。");
    } else {
        println!("文件已达到 {} MB，无需填充。", current_size / 1024 / 1024);
    }
    Ok(())
}
