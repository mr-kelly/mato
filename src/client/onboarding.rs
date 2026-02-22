use std::io::{self, Write};

// Embed templates at compile time
const POWER_USER: &str = include_str!("../../templates/power-user.json");
const SOLO_DEVELOPER: &str = include_str!("../../templates/solo-developer.json");
const ONE_PERSON_COMPANY: &str = include_str!("../../templates/one-person-company.json");
const FULLSTACK_DEVELOPER: &str = include_str!("../../templates/fullstack-developer.json");
const DATA_SCIENTIST: &str = include_str!("../../templates/data-scientist.json");
const MINIMAL: &str = include_str!("../../templates/minimal.json");

pub fn show_onboarding() -> io::Result<()> {
    println!("\n🎉 Welcome to Mato! 🎉\n");
    println!("Multi-Agent Terminal Office - Your persistent terminal multiplexer\n");
    
    println!("Choose a workspace template:\n");
    println!("  1. Mato Creator Office (45 tasks, 250+ tabs) ⭐ RECOMMENDED");
    println!("     → The actual office used by Mato creators/builders");
    println!();
    println!("  2. Solo Developer (3 tasks, 8 tabs)");
    println!("     → Perfect for individual developers");
    println!();
    println!("  3. One-Person Company (4 tasks, 13 tabs)");
    println!("     → Organized by business departments");
    println!();
    println!("  4. Full-Stack Developer (4 tasks, 11 tabs)");
    println!("     → Multiple projects + DevOps + Learning");
    println!();
    println!("  5. Data Scientist (4 tasks, 11 tabs)");
    println!("     → Data analysis, ML training, pipelines");
    println!();
    println!("  6. Minimal (1 task, 1 tab)");
    println!("     → Start from scratch");
    println!();
    
    print!("Enter your choice (1-6): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let choice = input.trim();
    let template_content = match choice {
        "1" => POWER_USER,
        "2" => SOLO_DEVELOPER,
        "3" => ONE_PERSON_COMPANY,
        "4" => FULLSTACK_DEVELOPER,
        "5" => DATA_SCIENTIST,
        "6" => MINIMAL,
        _ => {
            println!("\n❌ Invalid choice. Using minimal template.");
            MINIMAL
        }
    };
    
    // Write template to state file
    let state_path = crate::utils::get_state_file_path();
    if let Some(parent) = state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&state_path, template_content)?;
    
    println!("\n✅ Template applied successfully!");
    println!("📝 Your workspace is ready at: {}", state_path.display());
    println!("\n🚀 Starting Mato...\n");
    
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(())
}
