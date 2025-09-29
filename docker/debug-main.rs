use std::io::Write;

fn main() {
    println!("🔍 Debug: Starting application");
    std::io::stdout().flush().unwrap();
    
    println!("🔍 Debug: This is a simple test");
    std::io::stdout().flush().unwrap();
    
    // Just exit after printing
    println!("🔍 Debug: Exiting now");
    std::io::stdout().flush().unwrap();
}