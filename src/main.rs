use std::{env, io::{Read, Write}, net::{Shutdown, TcpListener, TcpStream}, sync::{Arc, Mutex}, thread};
use byteorder::{ByteOrder, LittleEndian};
use rust_glyph_recog::glyph_recognizer::GlyphRecognizer;

// todo: Make a client (for testing)
// todo: Make a server (for laod balancer)
// todo: Make a load balancer (for C# project)

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return;
    }
    match args[1].as_str() {
        "server" => start_as_server(),
        _ => (),
    }
}

fn start_as_server() {
    let input_dir = "dats/";
    let m = Arc::new(GlyphRecognizer::new_from_data_dir(input_dir));

    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let m = Arc::clone(&m);
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream, m);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}

fn handle_client(mut stream: TcpStream, recog: Arc<GlyphRecognizer>) {
    // read 2 bytes for glyph count
    // if 0 then do special load balancing thing (send 1 byte of all 0s back)
    // for i in 0..glyph count make a ray and parse it
    //      Send result back
    let mut glyph_count_buffer = [0 as u8; 2];
    match stream.read(&mut glyph_count_buffer) {
        Ok(_) => {
            let glyph_count = LittleEndian::read_u16(&glyph_count_buffer);
            println!("Client sending {} glyphs", glyph_count);
            let mut reply = Vec::new();
            for _ in 0..glyph_count {
                println!("Attempting to get and parse glyph");
                let result = recog.parse_glyph_from_stream(&mut stream);
                println!("Server parsed client data as {}", result);
                reply.extend(result.as_bytes());
            }
            stream.write(&[reply.len() as u8]).unwrap();
            stream.write(&reply).unwrap();
        },
        Err(_) => println!("Failed to get glyph count"),
    }
    stream.shutdown(Shutdown::Both).unwrap();
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::{fs::{self, File}, io::{self, Read, Write}, net::TcpStream, path::PathBuf, thread};
    use byteorder::ByteOrder;
    use rust_glyph_recog::glyph_recognizer::GlyphRecognizer;

    use crate::start_as_server;

    #[test]
    fn test_server() {
        println!("Test message");
        let server = thread::spawn(|| { println!("Test message 2"); start_as_server(); });
        println!("Test message 3");
        thread::sleep(time::Duration::from_secs(5));


        match TcpStream::connect("localhost:3333") {
            Ok(mut client) => {
                let mut buffer = [0u8; 2];
                byteorder::LittleEndian::write_u16(&mut buffer, 3u16);
                client.write(&buffer).unwrap();

                for i in 0..3 {
                    println!("Writing file {}", i);
                    let mut file = File::open(format!("dats/61/{}.dat", i)).unwrap();
                    io::copy(&mut file, &mut client).unwrap();
                }

                thread::sleep(time::Duration::from_secs(5));
                let mut buffer = vec![0u8; 1];
                client.read_exact(&mut buffer).unwrap();
                let bytes_expected = u8::from_le(buffer[0]);
                let mut buffer = vec![0u8; bytes_expected as usize];
                client.read_exact(&mut buffer).unwrap();
                println!("Got {} characters back. {}", bytes_expected, String::from_utf8(buffer).unwrap());
            },
            Err(e) => {
                panic!("Failed to connect: {}:", e);
            }
        }

        server.join().unwrap();
    }

    #[test]
    fn test_lib() {
        let input_dir = "dats/";
        let recog = GlyphRecognizer::new_from_data_dir(input_dir);

        let dirs:Vec<PathBuf> = fs::read_dir(input_dir).unwrap()
            .filter(|x| x.as_ref().unwrap().path().is_dir())
            .map(|x| x.unwrap().path())
            .collect();


        for dir in dirs.into_iter().filter(|dir| dir.file_name().unwrap().to_str().unwrap() != "overlaps") {
            // //for dir in dirs {
            let dir_name = dir.file_name().unwrap().to_str().unwrap().to_owned();
            let c = std::char::from_u32(dir_name.parse::<u32>().unwrap()).unwrap().to_string();
            for file in fs::read_dir(input_dir.to_owned() + dir.file_name().unwrap().to_str().unwrap()).unwrap().into_iter().map(|x| x.unwrap()) {
                let mut file = File::open(format!("{}{}/{}", input_dir, dir_name, file.path().file_name().unwrap().to_str().unwrap())).unwrap();
                let result = recog.parse_glyph_from_stream(&mut file);
                assert_eq!(result, c);
                //println!("Expected {} got {}", c, result);
            }
        }
    }
}
