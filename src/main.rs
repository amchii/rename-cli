use clap::Parser;
use colored::*;
use glob::Pattern;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = false)]
struct Args {
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg()]
    pattern: Option<String>,

    #[arg()]
    from_str: Option<String>,

    #[arg()]
    to_str: Option<String>,

    #[arg(short, long, help = "跳过最终确认，直接执行重命名")]
    yes: bool,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("{} {}", "Error:".red(), e);
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = &args.path;
    if !path.is_dir() {
        return Err(format!("'{}' 不是一个有效的目录。", path.display()).into());
    }

    // --- 1. 列出文件 ---
    println!("List {}:", path.display());
    let all_files = list_files_in_dir(path)?;
    if all_files.is_empty() {
        println!("目录 '{}' 为空或不包含文件。", path.display());
        return Ok(());
    }
    // 仅在交互模式下全部列出，非交互模式下会直接显示匹配结果
    if args.pattern.is_none() {
        for file_name in &all_files {
            println!("{}", file_name);
        }
    }

    // --- 2. 获取模式和替换字符串 ---
    let pattern_str: String;
    let from_str: String;
    let to_str: String;

    // 检查是进入交互模式还是非交互模式
    if let (Some(p), Some(f), Some(t)) = (args.pattern, args.from_str, args.to_str) {
        // 非交互模式
        pattern_str = p;
        from_str = f;
        to_str = t;
        println!("{}", "---------------------------------------------".yellow());
        println!("模式: {}", pattern_str.cyan());
        println!("替换: '{}' -> '{}'", from_str.cyan(), to_str.cyan());
    } else {
        // 交互模式
        println!("{}", "---------------------------------------------".yellow());
        print!("Filter pattern(Glob): ");
        io::stdout().flush()?;
        let mut p_input = String::new();
        io::stdin().read_line(&mut p_input)?;
        pattern_str = p_input.trim().to_string();

        if pattern_str.is_empty() {
            println!("未输入筛选模式，操作已取消。");
            return Ok(());
        }

        // 交互模式下获取替换字符串
        println!("{}", "---------------------------------------------".yellow());
        println!("Replace <A> to <B>:\n");
        print!("A: ");
        io::stdout().flush()?;
        let mut f_input = String::new();
        io::stdin().read_line(&mut f_input)?;
        from_str = f_input.trim().to_string();

        if from_str.is_empty() {
            println!("要被替换的字符串 <A> 不能为空。");
            return Ok(());
        }

        print!("B: ");
        io::stdout().flush()?;
        let mut t_input = String::new();
        io::stdin().read_line(&mut t_input)?;
        to_str = t_input.trim().to_string();
    }

    // --- 3. 筛选文件 ---
    let pattern = Pattern::new(&pattern_str)?;
    let matched_files: Vec<String> = all_files
        .into_iter()
        .filter(|file_name| pattern.matches(file_name))
        .collect();

    if matched_files.is_empty() {
        println!("\n没有文件匹配模式 '{}'", pattern_str);
        return Ok(());
    }

    // --- 4. 预览和确认 ---
    println!("\n{}", "匹配到的文件及重命名预览:".bold());
    let renames: Vec<(String, String)> = matched_files
        .iter()
        .map(|old_name| (old_name.clone(), old_name.replace(&from_str, &to_str)))
        .filter(|(old, new)| old != new) // 只处理实际发生变化的文件
        .collect();

    if renames.is_empty() {
        println!("没有需要重命名的文件。");
        return Ok(());
    }

    for (old, new) in &renames {
        println!("{} {} {}", old.red(), "->".yellow(), new.green());
    }

    let mut confirmation = String::new();
    if !args.yes {
        print!("\n是否继续? (y/N): ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut confirmation)?;
    }

    // --- 5. 执行重命名 ---
    if args.yes || confirmation.trim().to_lowercase() == "y" {
        println!("\n开始执行重命名...");
        for (old_name, new_name) in &renames {
            let old_path = path.join(old_name);
            let new_path = path.join(new_name);
            match fs::rename(&old_path, &new_path) {
                Ok(_) => println!("Renamed: {} -> {}", old_path.display(), new_path.display()),
                Err(e) => eprintln!("Failed to rename {}: {}", old_path.display(), e),
            }
        }
        println!("\n{} 重命名完成。", "Success:".green());
    } else {
        println!("操作已取消。");
    }

    Ok(())
}


fn list_files_in_dir(path: &Path) -> Result<Vec<String>, io::Error> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(file_name) = entry.file_name().to_str() {
                files.push(file_name.to_string());
            }
        }
        if files.len() >= 50 {
            break;
        }
    }
    files.sort();
    Ok(files)
}
