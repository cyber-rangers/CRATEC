use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::SinkExt;
use tokio::time::{sleep, Duration};
use crate::dashboard_layout::{get_device_status, DeviceUpdate};
use serde_json::json;

// Globální seznam klientů chráněný asynchroním mutexem
static CLIENTS: Lazy<Arc<Mutex<Vec<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

/// Najde volný port v zadaném rozsahu (včetně) a vrátí jej.
async fn find_available_port(start: u16, end: u16) -> Result<u16, String> {
    for port in start..=end {
        let addr = format!("127.0.0.1:{}", port);
        match TcpListener::bind(&addr).await {
            Ok(listener) => {
                // Uvolníme listener, abychom jej později mohli znovu použít v accept_loop.
                drop(listener);
                log::debug!("Port {} je volný", port);
                return Ok(port);
            }
            Err(e) => {
                log::debug!("Port {} není dostupný: {}", port, e);
                continue;
            }
        }
    }
    Err("Žádný port není dostupný v rozsahu 8080-8100".into())
}

/// Spustí websocket server. Najde volný port a spustí accept a broadcast smyčky.
/// Vrátí adresu websocketu (např. "ws://127.0.0.1:8080").
#[tauri::command]
pub async fn start_websocket_server() -> Result<String, String> {
    log::debug!("Spouštím start_websocket_server...");
    let port = find_available_port(8080, 8100).await?;
    let addr = format!("127.0.0.1:{}", port);
    let clients = CLIENTS.clone();
    
    // Spustíme accept loop s dynamicky vybranou adresou
    tokio::spawn(accept_loop(addr.clone(), clients.clone()));
    // Spustíme broadcast loop – každé 2 sekundy odešle full update
    tokio::spawn(broadcast_loop(clients.clone()));
    
    log::debug!("Websocket server spuštěn na {}", addr);
    Ok(format!("ws://{}", addr))
}

/// Accept loop: přijímá nová websocket spojení a přidává je do globálního seznamu.
async fn accept_loop(addr: String, clients: Arc<Mutex<Vec<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>>) {
    let listener = TcpListener::bind(&addr).await.map_err(|e| e.to_string()).unwrap();
    println!("Websocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let clients_clone = clients.clone();
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    log::debug!("Nové websocket spojení navázáno.");
                    clients_clone.lock().await.push(ws_stream);
                }
                Err(e) => {
                    eprintln!("WebSocket handshake failed: {}", e);
                }
            }
        });
    }
}

/// Broadcast loop: každé 2 sekundy získá aktuální stav (full update) a odešle ho všem klientům.
async fn broadcast_loop(clients: Arc<Mutex<Vec<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>>) {
    loop {
        // Získáme aktuální stav – předpokládáme, že get_device_status() vrací např. strukturu DeviceStatus
        let current_status = match get_device_status() {
            Ok(status) => status,
            Err(e) => {
                eprintln!("Error getting device status: {}", e);
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        // Vytvoříme JSON objekt se zadaným typem "Status" místo "Full"
        let status_update = json!({
            "type": "Status",
            "data": current_status
        });

        // Serializujeme zprávu
        let json_str = status_update.to_string();
        
        // Důležité: Držíme lock jen po dobu odeslání zpráv
        {
            let mut clients_lock = clients.lock().await;
            let mut indices_to_remove = Vec::new();
            for (i, client) in clients_lock.iter_mut().enumerate() {
                if let Err(e) = client.send(Message::Text(json_str.clone().into())).await {
                    eprintln!("Error sending update: {}", e);
                    indices_to_remove.push(i);
                }
            }
            // Odstraníme klienty, u kterých odeslání selhalo
            for i in indices_to_remove.iter().rev() {
                clients_lock.remove(*i);
            }
        } // Lock je zde uvolněn, když clients_lock jde mimo scope
        
        // Spánek probíhá až po uvolnění zámku
        sleep(Duration::from_secs(2)).await;
    }
}
/// Funkce, kterou můžete volat z jiných modulů pro odeslání libovolné zprávy.
pub async fn broadcast_message(msg: &str) {
    let clients = CLIENTS.clone();
    let mut clients_lock = clients.lock().await;
    let mut indices_to_remove = Vec::new();
    for (i, client) in clients_lock.iter_mut().enumerate() {
        if let Err(e) = client.send(Message::Text(msg.into())).await {
            eprintln!("Error sending message: {}", e);
            indices_to_remove.push(i);
        }
    }
    for i in indices_to_remove.iter().rev() {
        clients_lock.remove(*i);
    }
}
