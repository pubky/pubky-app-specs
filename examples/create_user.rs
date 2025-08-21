/// cargo run --example create_user

#[cfg(target_arch = "wasm32")]
fn main() {
    // Do nothing when compiled for wasm
}

#[cfg(not(target_arch = "wasm32"))]
use {
    anyhow::Result,
    pubky::Client,
    pubky::{Keypair, PublicKey},
    pubky_app_specs::{
        traits::{HasPath, Validatable},
        PubkyAppUser, PROTOCOL,
    },
    serde_json::to_vec,
};
// Replace this with your actual homeserver public key

#[cfg(not(target_arch = "wasm32"))]
const HOMESERVER: &str = "ufibwbmed6jeq9k4p583go95wofakh9fwpp4k734trq79pd9u1uy";

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<()> {
    // Print an introduction for the developer
    println!("Welcome to the Pubky User Creator Example!");

    // Step 1: Initialize the Pubky client
    println!("\nStep 1: Initializing the Pubky client...");

    let client = Client::builder().build()?;
    let homeserver = PublicKey::try_from(HOMESERVER).expect("Invalid homeserver public key.");

    println!("Pubky client initialized successfully.");

    // Step 2: Generate a keypair for the new user
    println!("\nStep 2: Generating a random keypair for the new user...");

    let keypair = Keypair::random();
    let user_id = keypair.public_key().to_z32();

    println!("Generated keypair with User ID: {}", user_id);

    // Step 3: Sign up a new identity on the homeserver
    println!("\nStep 3: Signing up the new identity on the homeserver...");

    // This step will likely fail "as is" as homeservers are no requiring sign up tokens.
    // Here we provide `None` signup token.
    client
        .signup(&keypair, &homeserver, None)
        .await
        .expect("Failed to sign up the user on the homeserver.");

    println!("User signed up successfully!");

    // Step 4: Create a new user profile
    println!("\nStep 4: Creating a new user profile...");

    let user_profile = PubkyAppUser::new(
        "Test User".to_string(), // User display name
        None,                    // Optional fields set to None
        None,
        None,
        None,
    );

    println!("User profile created: {:?}", user_profile);

    // Step 5: Write the user profile to the homeserver
    println!("\nStep 5: Writing the user profile to the homeserver...");

    let url = format!(
        "{protocol}{pubky_id}{path}",
        protocol = PROTOCOL,
        pubky_id = user_id,
        path = PubkyAppUser::create_path()
    );
    let content = to_vec(&user_profile)?;

    client
        .put(url.as_str())
        .body(content.clone())
        .send()
        .await
        .expect("Failed to write the user profile to the homeserver.");

    println!(
        "User profile written successfully to:\nURL: {}\nContent: {}",
        url,
        String::from_utf8_lossy(&content)
    );

    // Step 6: Retrieve the user profile from the homeserver
    println!("\nStep 6: Retrieving the user profile from the homeserver...");

    let response = client
        .get(url.as_str())
        .send()
        .await
        .expect("Failed to retrieve the user profile from the homeserver.");

    let retrieved_content = response.bytes().await?;

    let retrieved_profile = <PubkyAppUser as Validatable>::try_from(&retrieved_content, "")
        .expect("Failed to deserialize the retrieved user profile.");

    println!(
        "User profile retrieved successfully:\n{}",
        serde_json::to_string_pretty(&retrieved_profile).unwrap()
    );

    // Final message to indicate completion
    println!("\nAll steps completed successfully! The new user is now registered and their profile is stored on the homeserver.");

    Ok(())
}
