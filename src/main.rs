////////////////////
// Rustcat (rc) 
// by: robiot
////////////////////

//Todo: Replace unwrap
use termion::color;
use getopts::Options;
use std::io::Write;


/* Global Variables */
const  __VERSION__: &'static str = env!("CARGO_PKG_VERSION");

/* Use laters */
struct Opts<'a> {
    host: &'a str,
    port: &'a str,
    transport: Transport
}

enum Transport {
    Tcp,
    Udp,
}

/* Help -h */
fn print_help(program: &str, opts: Options, code: i32) {
    let brief = format!("Usage: {} [options] [destination] [port]", program);
    print!("{}", opts.usage(&brief));
    if code != 0 {
        std::process::exit(code);
    }
}

/* Prints error */
fn print_error(err: &str) {
    eprintln!("{}rc:{} {}", color::Fg(color::LightRed), color::Fg(color::Reset), err);
}

/* Piped thread */
fn pipe_thread<R, W>(mut r: R, mut w: W) -> std::thread::JoinHandle<()>  where R: std::io::Read + Send + 'static, W: std::io::Write + Send + 'static
{
    std::thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            let len = r.read(&mut buffer).unwrap();
            if len == 0 {
                println!("Connection lost");
                std::process::exit(0x0100);
            }
            w.write(&buffer[..len]).unwrap();
            w.flush().unwrap();
        }
    })
}

/* Listen on given host and port */
fn listen(opts: &Opts) -> std::io::Result<()>{
    println!("Listening on {}{}{}:{}{}{}", color::Fg(color::LightGreen), opts.host, color::Fg(color::Reset), color::Fg(color::LightCyan), opts.port, color::Fg(color::Reset)); //move this?

    match std::io::stdout().flush() {
        Ok(m) => m,
        Err(err) => {
            return Err(err);
        }
    };

    match opts.transport {
        Transport::Tcp => {
            let listener = match std::net::TcpListener::bind((opts.host, opts.port.parse::<u16>().expect("Not a valid port"))) { // todo: Better handling if not a valid number given
                Ok(m) => m,
                Err(err) => {
                    return Err(err)
                }
            };
            
            let (stream, _) = listener.accept().unwrap();
            let t1 = pipe_thread(std::io::stdin(), stream.try_clone().unwrap());
            let t2 = pipe_thread(stream, std::io::stdout());
            t1.join().unwrap();
            t2.join().unwrap();
        }

        Transport::Udp => {
            //todo: add udp alternative
        }
    }
    return Ok(());
}

/* Main */
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "This help text");
    opts.optflag("v", "version", "Application Version");
    opts.optflag("l", "", "Listen mode");
    opts.optflag("p", "", "Listen port");
    opts.optflag("u", "", "UDP mode (Not available yet)");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => {
            print_error(&err.to_string());
            return;
        }
    };

    if matches.opt_present("h") {
        print_help(&program, opts, 0);
        return;
    } else if matches.opt_present("v") {
        println!("Rustcat v{}",__VERSION__);
        return;
    }
    
    // If any arguement given
    let (opt_host, opt_port) = if matches.free.len() == 1 && matches.opt_present("l") && matches.opt_present("p"){
        ("0.0.0.0", matches.free[0].as_str())
    } else if matches.free.len() == 2 && matches.opt_present("l"){
        (matches.free[0].as_str(), matches.free[1].as_str())
    }
    else {
        print_help(&program, opts, 1);
        ("","")
    };

    let opts = Opts {
        host: opt_host,
        port: opt_port,
        transport: if matches.opt_present("u") {
            Transport::Udp
        } else {
            Transport::Tcp
        }
    };

    if let Err(err) = listen(&opts) {
        print_error(&err.to_string());
        return;
    };
}