use rayon::ThreadPoolBuilder;

/// Configure the global Rayon thread pool.
pub fn configure_threads(threads: Option<usize>) -> anyhow::Result<()> {
    if let Some(n) = threads {
        ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .map_err(|e| anyhow::anyhow!("failed to configure thread pool: {e}"))?;
    }
    Ok(())
}
