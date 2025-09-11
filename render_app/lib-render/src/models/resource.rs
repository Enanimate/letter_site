use crate::texture;


pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    println!("file: {file_name}");
    let data = load_binary(file_name).await?;
    println!("Flag 5");
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    let data = {
        let url = format_url(file_name);
        reqwest::get(url).await?.bytes().await?.to_vec()
    };
    #[cfg(not(target_arch = "wasm32"))]
    let data = {
        use std::env;

        let mut path = env::current_dir()?;
            path.push("resources");
            path.push(file_name);
            println!("{path:#?}");
        std::fs::read(path)?
    };

    Ok(data)
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    #[cfg(target_arch = "wasm32")]
    let text = {
        let url = format_url(file_name);
        reqwest::get(url).await?.text().await?
    };

    #[cfg(not(target_arch = "wasm32"))]
    let text = {
        use std::env;

        let mut path = env::current_dir()?;
        path.push("resources");
        path.push(file_name);
        println!("{path:#?}");
        std::fs::read_to_string(path).unwrap()
    };

    Ok(text)
}

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    
    if !origin.ends_with("resources") {
        origin = format!("{}/resources", origin);
    }

    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}