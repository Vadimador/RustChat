// État des demandes du prof pour le serveur :
//  - ownership : tchek
//  - borrowing : tchek
//  - des collections : theck, il y a un vecteur
//  - des tests : X
//  - de la propagation d'erreur : bof
//  - des structures : nope 
//  - des enums : ... non plus
//  - des threads : OUI !

use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self};
use std::thread;
use std::str;
use magic_crypt::MagicCryptTrait;
use magic_crypt::new_magic_crypt;

const LOCAL: &str = "127.0.0.1:6000";


// test connexion tcpStream 
#[cfg(test)]
mod tests {
    #[test]
    fn test_tcpstream() {
        use std::net::TcpStream;        
        const LOCAL: &str = "127.0.0.1:6000";
        let _test = TcpStream::connect(LOCAL).expect("Le Stream n'a pas reussi a se connecter");
    }
}


const MSG_SIZE: usize = 100;

fn main() {
    println!("lancement d'un client");
    let mut client = TcpStream::connect(LOCAL).expect("Le Stream n'a pas réussi à se connecter");
    client.set_nonblocking(true).expect("Echec de l'initialisation en mode non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    let mut name : String = "anonymous".to_owned();
    let mut name_connected = "✅ bruh".to_owned();
    let mut anon : bool = true;

    println!("Écrire un message:");
    thread::spawn( move || loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("la lecture à partir de stdin a échoué");
        tx.send(buff).expect("transmission du channel impossible");
    });

    loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).ok().unwrap();
                
                if !msg.is_empty() {
                    if msg.starts_with('!') && msg.chars().nth(1).unwrap() == '!' {
                        if msg.find("!!connected") != Option::None {
                            name_connected = String::from("✅ ");
                            let svec : Vec<&str> = msg.split(' ').collect();
                            name_connected.push_str(svec[1]);
                            anon = false;
                            println!("Vous êtes connecté en tant que \"{}\"",svec[1]);
                        }
                    }
                    else {
                        println!("{:?}",msg);
                    } 
                 }
                
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("la connexion avec le serveur a été coupée !");
                break;
            }
        }

        if let Ok(buff) = rx.try_recv() {
            // ----------------------------------------------- Test pour savoir les commandes
            if buff.starts_with(':') {        
                if buff.find(":name") != Option::None { // =========================== changer de pseudo quand on est anonymous
                    let svec : Vec<&str> = buff.split(' ').collect();
                    name = svec[1].trim().to_owned();
                    println!("Votre nouveau nom d'anonyme est : {}",name.as_str());
                }
                else if buff.find(":new_account") != Option::None{ // ================ créer un nouvel account
                    let svec : Vec<&str> = buff.split(' ').collect();
                    let account_name = svec[1].trim().to_owned();
                    let account_mdp = svec[2].trim().to_owned();

                    let mut msg : String = String::from("!!create ");
                    msg.push_str(account_name.trim());
                    msg.push(' ');
                    msg.push_str(account_mdp.trim());
                    send(&mut client, msg);
                }
                else if buff.find(":connect") != Option::None{ // =================== se connecter à un compte déjà crée
                    let svec : Vec<&str> = buff.split(' ').collect();
                    let account_name = svec[1].trim().to_owned();
                    let account_mdp = svec[2].trim().to_owned();

                    let mut msg : String = String::from("!!connect ");
                    msg.push_str(account_name.trim());
                    msg.push(' ');
                    msg.push_str(account_mdp.trim());
                    send(&mut client, msg);
                }
                else if buff.find(":quit") != Option::None {
                    if !anon {
                        anon = true;
                        println!("Vous vous êtes déconnecté.");
                    }
                    else {
                       break; 
                    }
                    
                }
                else {
                    eprintln!("/!\\ La commande proposé n'existe pas.");
                }
            }
            else {
                let mc = new_magic_crypt!("magickey", 256);
                let mut msg : String;
                if anon {
                    msg = name.clone();
                }
                else {
                    msg = name_connected.clone();
                }
                msg.push_str(" : ");
                msg.push_str(buff.trim());
                let ciphertext = mc.encrypt_str_to_base64(&msg);
                send(&mut client,ciphertext);
            }
        }
    }
    println!("Aurevoir, à bientôt!");

}

fn send(client : &mut std::net::TcpStream, msg : String) {
    let mut msg = msg.into_bytes();
    msg.resize(MSG_SIZE, 0);
    client.write_all(&msg).expect("l'écriture sur le socket a échoué");
}