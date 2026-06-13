use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::Path;

fn main() -> io::Result<()> {
    println!("=== PAK 文件大小调整工具 ===\n");

    // 选择模式
    let mode = loop {
        println!("请选择操作：");
        println!("  1. 放大（填充到 500 MB）");
        println!("  2. 缩小（移除末尾无意义填充，恢复原始大小）");
        print!("输入数字 (1/2): ");
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        match choice.trim() {
            "1" => break "enlarge",
            "2" => break "shrink",
            _ => println!("无效输入，请重新输入。\n"),
        }
    };

    // 拖入文件
    print!("\n请将 PAK 文件拖入此窗口，然后按回车：");
    io::stdout().flush()?;
    let mut path_input = String::new();
    io::stdin().read_line(&mut path_input)?;
    let path = path_input.trim().trim_matches('"');
    if path.is_empty() || !Path::new(path).exists() {
        eprintln!("错误：文件不存在。");
        wait_for_exit();
        return Ok(());
    }

    // 获取当前文件大小
    let metadata = fs::metadata(path)?;
    let current_size = metadata.len();
    let current_mb = current_size as f64 / (1024.0 * 1024.0);
    println!("\n当前文件大小: {} 字节 (≈{:.2} MB)", current_size, current_mb);

    match mode {
        "enlarge" => {
            let target_mb = 500u64;
            let target_size = target_mb * 1024 * 1024;
            if current_size >= target_size {
                println!("文件已经 ≥ {} MB，无需放大。", target_mb);
                wait_for_exit();
                return Ok(());
            }

            println!("将把文件从 {:.2} MB 放大到 {} MB。", current_mb, target_mb);
            print!("确认操作？(y/N): ");
            io::stdout().flush()?;
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;
            if confirm.trim().to_lowercase() != "y" {
                println!("操作已取消。");
                wait_for_exit();
                return Ok(());
            }

            let file = OpenOptions::new().write(true).open(path)?;
            file.set_len(target_size)?;
            println!("\n✅ 放大完成！文件大小: {} MB", target_mb);
        }

        "shrink" => {
            // 从文件末尾反向查找第一个非零字节
            let original_size = find_original_size(path)?;
            if original_size == current_size {
                println!("\n未检测到尾部填充，文件大小未变，无需缩小。");
                wait_for_exit();
                return Ok(());
            }

            let original_mb = original_size as f64 / (1024.0 * 1024.0);
            println!("\n检测到原始数据结束位置: {} 字节 (≈{:.2} MB)", original_size, original_mb);
            println!("将移除尾部 {} 字节的填充。", current_size - original_size);
            print!("确认缩小？(y/N): ");
            io::stdout().flush()?;
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;
            if confirm.trim().to_lowercase() != "y" {
                println!("操作已取消。");
                wait_for_exit();
                return Ok(());
            }

            let file = OpenOptions::new().write(true).open(path)?;
            file.set_len(original_size)?;
            println!("\n✅ 缩小完成！文件已恢复为原始大小: {:.2} MB", original_mb);
        }
        _ => unreachable!(),
    }

    wait_for_exit();
    Ok(())
}

/// 从文件末尾查找第一个非零字节，返回原始内容应有的长度（即最后一个非零字节的位置 + 1）
fn find_original_size(path: &str) -> io::Result<u64> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let file_size = file.metadata()?.len();
    if file_size == 0 {
        return Ok(0);
    }

    // 一次读一块，从尾部向前扫描，避免把整个文件读入内存
    let mut buf = vec![0u8; 4096];
    let mut pos = file_size;

    while pos > 0 {
        let read_size = std::cmp::min(buf.len() as u64, pos) as usize;
        let seek_pos = pos - read_size as u64;
        file.seek(SeekFrom::Start(seek_pos))?;
        file.read_exact(&mut buf[..read_size])?;

        // 从当前块的末尾向前找非零字节
        for i in (0..read_size).rev() {
            if buf[i] != 0 {
                return Ok(seek_pos + i as u64 + 1);
            }
        }
        pos = seek_pos;
    }
    // 整个文件全是零，返回 0（即空文件）
    Ok(0)
}

fn wait_for_exit() {
    print!("\n按回车键退出...");
    io::stdout().flush().ok();
    let _ = io::stdin().read_line(&mut String::new());
}
