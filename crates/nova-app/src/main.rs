use nova_app::NovaApp;

fn main() {
    println!("Starting NOVA 2.0 Gaming Platform & Engine...");
    let mut app = NovaApp::new();

    println!("Catalog loaded: {} experiences registered.", app.backend.catalog.len());
    for exp in app.backend.catalog.values() {
        println!(" - Catalog Item: [{}] {} (Genre: {})", exp.id, exp.title, exp.genre);
    }

    let first_exp_id = *app.experience_places.keys().next().unwrap();
    println!("Launching first experience: {}", first_exp_id);
    app.launch_by_id(first_exp_id);

    println!("Experience launched into shared runtime. Simulating initial ticks...");
    for _ in 0..10 {
        app.tick(0.016);
    }
    println!("NOVA runtime active and operating cleanly with live session frame count: {}",
        app.active_session.as_ref().unwrap().engine_time.frame_count);
}
