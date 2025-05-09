mod cratis_config;

/// ```
fn main() {
    // Get this via an initial command from user after configuring cratis.yml
    cratis_config::load_config("cratis.yml");
    
    let config = cratis_config::get_config();
    println!("Client ID: {}", config.client.id);
    println!("Backup mode: {:?}", config.backup.mode);
}
