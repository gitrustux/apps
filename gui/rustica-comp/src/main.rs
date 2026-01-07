//! Rustica Compositor
//!
//! Wayland compositor built with Smithay for the Rustica OS desktop environment.

use anyhow::Result;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

mod compositor;
mod state;

use compositor::RusticaCompositor;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    info!("Starting Rustica Compositor");

    // Create and run compositor
    match RusticaCompositor::new() {
        Ok(mut compositor) => {
            info!("Compositor initialized successfully");
            compositor.run();
        }
        Err(e) => {
            error!("Failed to initialize compositor: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
