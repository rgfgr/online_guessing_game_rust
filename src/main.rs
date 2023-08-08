use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    cmp::Ordering,
};
use online_guessing_game::ThreadPool;
use rand::Rng;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    println!("{request_line}");

    let request_parts: Vec<&str> = request_line.split_whitespace().collect();

    println!("{:#?}", request_parts);

    let (page, params) = manage_params(request_parts[1]);

    let request_line = format!(
        "{} {page} {}", request_parts[0], request_parts[2]
    );

    let (status_line, filename, error) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hi.html", false),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html", true),
    };

    let mut contents = fs::read_to_string(filename).unwrap();

    if !error {
        contents = create_page(contents, params);
    }

    let length = contents.len();

    let response = format!(
        "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"
    );
    stream.write_all(response.as_bytes()).unwrap();
}

fn create_page(contents: String, params: Vec<&str>) -> String {
    let mut content_lines: Vec<&str> = contents.split("\r\n").collect();
    let guess: u32 = match params[0].trim().parse() {
        Ok(num) => num,
        Err(_) => u32::MAX,
    };
    let number: u32 = match params[1].trim().parse() {
        Ok(num) => num,
        Err(_) => u32::MAX,
    };
    let number = number >> 2;
    let mut number_text: String = "test".to_string();
    let (mut game_text, mut win, mut failed) = ("Guess a number between 1 and 100", false, true);

    if guess == u32::MAX || number >= (u32::MAX >> 2) {
        let number: u32 = rand::thread_rng().gen_range(1..=100);
        number_text = (number << 2).to_string();
    } else {
        (game_text, win) = guessing_game(guess, number);
        failed = false;
    }

    let line_17 = format!(
        "let collection = [\"{}\", \"{}\", {}, {}]", game_text, number_text, win, failed
    );

    println!("{line_17}");

    content_lines[16] = &line_17;

    content_lines.join("\r\n")
}

fn guessing_game(guess: u32, number: u32) -> (&'static str, bool) {
    match guess.cmp(&number) {
        Ordering::Less => {
            println!("Too small!");
            ("Too small!", false)
        }
        Ordering::Greater => {
            println!("Too big!");
            ("Too big!", false)
        }
        Ordering::Equal => {
            println!("You win!");
            ("You win!", true)
        }
    }
}

fn manage_params(parms_part: &str) -> (&str, Vec<&str>) {
    let content: Vec<&str> = parms_part.split('?').collect();
    let page = content[0];
    let mut param_vals: Vec<&str> = ["temp", "temp"].to_vec();

    if content.len() < 2 {
        return (page, param_vals);
    }

    for param in content[1].split('&') {
        let param_parts: Vec<&str> = param.split('=').collect();
        let (name, val) = (param_parts[0], param_parts[1]);
        match name {
            "guess" => param_vals[0] = val,
            "number" => param_vals[1] = val,
            _ => continue,
        }
    }

    (page, param_vals)
}
