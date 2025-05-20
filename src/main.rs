use clap::Parser;
use zbus::Connection;
mod user;
mod accounts;
use crate::user::UserProxy;
use crate::accounts::AccountsProxy;

#[derive(Parser)]
#[command(name = "agreeter")]
#[command(about = "Yet another greetd greeter", long_about = None)]
struct Cli {
    /// List all cached users
    #[arg(short, long)]
    list_users: bool,
}

async fn list_users(connection: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let proxy = AccountsProxy::new(connection).await?;

    match proxy.list_cached_users().await {
        Ok(users) => {
            if users.is_empty() {
                println!("No cached users found.");
            } else {
                for path in users {
                    let user = UserProxy::builder(connection)
                        .path(path)?
                        .build()
                        .await?;
                    println!("{}", user.user_name().await?);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to list users: {}", e);
            return Err(Box::new(e));
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let connection = Connection::system().await?;

    if cli.list_users {
        list_users(&connection).await?;
    }

    Ok(())
}
