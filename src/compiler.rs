pub fn compile(source: String) -> String {
    let mut c_globals = format!(
        "#include <stdio.h>\n#include <stdlib.h>\n{}\n\
        void _qa_clear() {{ int c; while ((c = getchar()) != '\\n' && c != EOF); }}\n\n",
        get_os_sleep_header()
    );
    let mut c_functions = String::new();
    let mut main_calls = String::new();
    let mut remaining = String::new();

    for part in source.split(';') {
        let t = part.trim();
        if t.starts_with("set") && !t.contains('{') {
            c_globals.push_str(&format!("{};\n", process_set(t.to_string())));
        } else {
            remaining.push_str(part); remaining.push(';');
        }
    }

    for block in remaining.split("fun ") {
        let t = block.trim();
        if t.is_empty() { continue; }
        if let Some(sb) = t.find('{') {
            if let Some(eb) = t.rfind("};") {
                process_function(t, sb, eb, &mut c_functions, &mut main_calls);
                continue;
            }
        }
        process_execs(t, &mut main_calls);
    }

    format!("{}{}\nint main() {{\n{}\n    return 0;\n}}", c_globals, c_functions, main_calls)
}

fn get_os_sleep_header() -> &'static str {
    if cfg!(windows) {
        "#include <windows.h>\n#define _qa_sleep(ms) Sleep(ms)"
    } else {
        "#include <unistd.h>\n#define _qa_sleep(ms) usleep((ms) * 1000)"
    }
}

fn process_set(c: String) -> String {
    let res = if c.starts_with("set 64") { c.replacen("set 64", "long long", 1) }
              else if c.starts_with("set 32") { c.replacen("set 32", "int", 1) }
              else { c.replacen("set", "int", 1) };
    fix_vars(&res)
}

pub fn fix_vars(c: &str) -> String {
    let mut words = Vec::new();
    let parts: Vec<&str> = c.split_whitespace().collect();
    let ops = ["=", "<", ">", "<=", ">=", "==", "+", "-", "*", "/", ","];
    for (i, word) in parts.iter().enumerate() {
        let clean = word.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-');
        if !clean.is_empty() && clean.chars().all(|ch| ch.is_numeric() || (ch == '-' && clean.len() > 1)) {
            if i > 0 && ops.contains(&parts[i-1]) { words.push(word.to_string()); }
            else { words.push(word.replace(clean, &format!("v{}", clean))); }
        } else { words.push(word.to_string()); }
    }
    words.join(" ")
}

fn process_function(t: &str, sb: usize, eb: usize, c_functions: &mut String, main_calls: &mut String) {
    let raw_head = t[..sb].trim();

    let head_c = if let Some(open) = raw_head.find('(') {
        let name = &raw_head[..open];
        let args_raw = &raw_head[open+1..raw_head.rfind(')').unwrap_or(raw_head.len())];
        let fixed_args = fix_vars(args_raw);
        
        let typed_args = if fixed_args.trim().is_empty() { 
            "void".to_string() 
        } else { 
            let mut s = format!("int {}", fixed_args);
            s = s.replace(",", ", int");
            s
        };
        format!("void {}({})", name, typed_args)
    } else {
        format!("void {}(void)", raw_head)
    };

    let mut body_c = String::new();
    for line in t[sb+1..eb].split('\n') {
        let mut cmd = line.trim().to_string();
        if cmd.is_empty() || cmd == ";" { continue; }
        
        if cmd.contains("if") || cmd.contains("elif") || cmd.contains("else") { cmd = process_logic(cmd); }
        if cmd.contains("loop {") { cmd = cmd.replace("loop {", "while (1) {"); }
        if cmd.contains("break") { cmd = "break".into(); }
        if cmd.contains("clear") { cmd = "_qa_clear()".into(); }
        if cmd.starts_with("sleep(") { cmd = cmd.replace("sleep(", "_qa_sleep("); }
        
        if cmd.starts_with("exec") {
            cmd = process_exec_line(cmd);
        } else if !cmd.contains('{') && !cmd.contains('}') {
            if cmd.starts_with("set") { cmd = process_set(cmd); }
            else if cmd.starts_with("print") { cmd = process_print(cmd); }
            else if cmd.starts_with("input") { cmd = process_input(cmd); }
            else { cmd = fix_vars(&cmd); }
        }

        if !cmd.is_empty() && !cmd.ends_with(';') && !cmd.ends_with('{') && !cmd.ends_with('}') {
            cmd.push(';');
        }
        body_c.push_str(&format!("    {}\n", cmd));
    }
    c_functions.push_str(&format!("{} {{\n{}}}\n\n", head_c, body_c));
    process_execs(&t[eb+2..], main_calls);
}

fn process_exec_line(cmd: String) -> String {
    let start = cmd.find('(').unwrap_or(0);
    let end = cmd.rfind(')').unwrap_or(cmd.len());
    let inner = &cmd[start+1..end];
    let call = fix_vars(inner);
    if let Some((f, a)) = call.split_once(',') { 
        format!("{}({})", f.trim(), a.trim())
    } else { 
        format!("{}()", call.trim())
    }
}

fn process_logic(c: String) -> String {
    let t = c.trim();
    if t.contains("if") || t.contains("elif") {
        let is_el = t.contains("elif");
        let kw = if is_el { "elif" } else { "if" };
        let cond_raw = t.split(kw).nth(1).unwrap().replace('{', "").trim().to_string();
        
        let mut f_cond = fix_vars(&cond_raw).replace(" and ", " && ").replace(" or ", " || ");
        if f_cond.contains('=') && !f_cond.contains(">=") && !f_cond.contains("<=") && !f_cond.contains("==") {
            f_cond = f_cond.replace("=", "==");
        }

        if is_el { format!("}} else if ({}) {{", f_cond) } else { format!("if ({}) {{", f_cond) }
    } else if t.contains("else") { "} else {".into() } else { c }
}

fn process_print(c: String) -> String {
    if let Some(s) = c.find("print(") {
        let content = &c[s+6..c.rfind(")").unwrap()];
        if content.contains('"') { format!("printf(\"%s\\n\", {});", content) }
        else { format!("printf(\"%lld\\n\", (long long)({}));", fix_vars(content)) }
    } else { c }
}

fn process_input(c: String) -> String {
    if let Some(s) = c.find("input(") {
        let var = c[s+6..c.rfind(")").unwrap()].trim();
        let f_var = if var.chars().all(|ch| ch.is_numeric()) { format!("v{}", var) } else { var.to_string() };
        format!("scanf(\"%lld\", &{})", f_var)
    } else { c }
}

fn process_execs(s: &str, m: &mut String) {
    for p in s.split(';') {
        let t = p.trim();
        if t.starts_with("exec") {
            let res = process_exec_line(t.to_string());
            m.push_str(&format!("    {};\n", res));
        }
    }
}
