pub mod commands {
    use std::net::IpAddr;

    use trippy_core::Builder;

    #[tauri::command]
    #[specta::specta]
    pub async fn run_traceroute(ip: IpAddr) -> Result<Vec<IpAddr>, String> {
        let tracer = Builder::new(ip)
            .max_rounds(Some(5))
            .build()
            .map_err(|e| e.to_string())?;

        tracer.run().map_err(|e| e.to_string())?;

        let hops = tracer
            .snapshot()
            .hops()
            .iter()
            .map(|hop| hop.addrs().copied())
            .flatten()
            .collect();

        Ok(hops)
    }
}
