use itertools::Itertools;
use std::collections::HashMap;
use std::io::BufRead;
use std::ops::Fn;

fn from_str_hex_to_u32(s: &str) -> u32 {
    u32::from_str_radix(s.strip_prefix("0X").unwrap(), 16).expect("Unable to parse integer")
}
fn from_str_hex_to_u8(s: &str) -> u8 {
    u8::from_str_radix(s.strip_prefix("0X").unwrap(), 16).expect("Unable to parse integer")
}

fn rc4_guess_m_for_iv(c: &HashMap<u32, u8>, f: impl Fn(u8, u8) -> u8) -> u8 {
    let freq = c
        .into_iter()
        .map(|(iv, c)| {
            let iv: u8 = (iv & 0xFF) as u8;
            let m = f(iv, *c);
            m
        })
        .into_group_map_by(|x| *x);

    freq.into_iter()
        .max_by(|x, y| x.1.len().cmp(&y.1.len()))
        .unwrap()
        .0
}

fn attack_rc4(data: Vec<HashMap<u32, u8>>) -> (u8, Vec<u8>) {
    let iv01 = data.get(0).unwrap();
    let m0 = rc4_guess_m_for_iv(iv01, |iv, c| c ^ (iv.wrapping_add(2)));

    let mut key = vec![];

    for n in 3..16 {
        let d = n * (n + 1) / 2;
        let iv_map = data.get((n - 2) as usize).unwrap();
        let kn = rc4_guess_m_for_iv(&iv_map, |x, c| {
            let k_acc = key.iter().fold(0u8, |acc, &k| acc.wrapping_sub(k));
            (c ^ m0).wrapping_sub(x).wrapping_sub(d).wrapping_add(k_acc)
        });

        key.push(kn);
    }

    (m0, key)
}

fn rc4_encrypt(key: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let mut s: Vec<u8> = (0..=255).collect();
    let key_len = key.len();

    let mut j = 0;
    for i in 0..256 {
        j = (j + s[i] as usize + key[i % key_len] as usize) % 256;
        s.swap(i, j);
    }

    let mut i = 0;
    j = 0;
    let mut ciphertext = Vec::with_capacity(plaintext.len());
    for &byte in plaintext {
        i = (i + 1) % 256;
        j = (j + s[i] as usize) % 256;
        s.swap(i, j);
        let k = s[(s[i] as usize + s[j] as usize) % 256];
        ciphertext.push(byte ^ k);
    }

    ciphertext
}

// Generate a key with the given IV format: (z, 0xFF, x, 3, 4, ..., 15)
fn generate_key_with_iv(z: u8, x: u8) -> (u32, [u8; 16]) {
    let mut key = [0u8; 16];

    // 3 byte IV
    key[0] = z;
    key[1] = 0xFF;
    key[2] = x;

    let my_key: &[u8] = b"CTF{whyweep}";

    for i in 3..16 {
        if i - 3 >= my_key.len() {
            key[i] = 0;
        } else {
            key[i] = my_key[i - 3];
        }
    }

    (((z as u32) << 16) | 0x00FF00 | (x as u32), key)
}

fn generate_iv_z_ff_x(z: u8) -> HashMap<u32, u8> {
    (0x00..=0xFF)
        .map(|x| {
            let (iv, key) = generate_key_with_iv(z, x);
            let plaintext = b"cry: helloo world!";
            let ciphertext = rc4_encrypt(&key, plaintext);
            (iv, ciphertext[0])
        })
        .collect::<HashMap<_, _>>()
}

fn save_data(hashmap: &Vec<HashMap<u32, u8>>, name: &str) {
    let mut content = String::new();
    hashmap.into_iter().for_each(|hm| {
        hm.into_iter()
            .sorted_by(|(a, _), (b, _)| a.cmp(b))
            .for_each(|(iv, c)| {
                content.push_str(&format!("0X{:06x} 0X{:02x}\n", iv, c));
            });
    });
    std::fs::write(name, content).expect("Unable to write file");
}

fn parse_synthetic_data(path: &str) -> Vec<HashMap<u32, u8>> {
    let file = std::fs::File::open(path).expect("Unable to open file");
    let file = std::io::BufReader::new(file);

    let s = file
        .lines()
        .map(|line| {
            let line = line.expect("Unable to read line");
            let line_parts = line.split(" ").collect::<Vec<&str>>();
            let iv = from_str_hex_to_u32(line_parts.get(0).unwrap());
            let cipher = from_str_hex_to_u8(line_parts.get(1).unwrap());
            (iv, cipher)
        })
        .chunk_by(|iv| iv.0 >> 16)
        .into_iter()
        .map(|chunk| chunk.1.into_iter().collect::<HashMap<u32, u8>>())
        .collect::<Vec<HashMap<u32, u8>>>();
    s
}

fn main() {
    assert_eq!(
        rc4_encrypt(b"Key", b"Plaintext"),
        vec![0xBB, 0xF3, 0x16, 0xE8, 0xD9, 0x40, 0xAF, 0x0A, 0xD3]
    );

    let synthetic_data = (1..=1)
        .chain(3..16)
        .map(|z| generate_iv_z_ff_x(z as u8))
        .collect::<Vec<_>>();

    assert_eq!(synthetic_data.len(), 14);
    save_data(&synthetic_data, "ctf.dat");
    let synthetic_data = parse_synthetic_data("ctf.dat");

    let (m0, k) = attack_rc4(synthetic_data);
    let s = k.iter().map(|b| *b as char).collect::<String>();
    println!("RC4 key: {:02x?}, {:02x?}", m0, k);
    println!("Key in string {}", s);
    assert_eq!(s, "CTF{whyweep}\0");
}
