use std::io::Write;
use std::io::Read;

struct Header {
    name: String,
    mode: String,
    uid: u16,
    gid: u16,
    mtime: u64,
    checksum: u16,
    typeflag: [char; 1],
    // linkname: [char; 100],
    // magic: [char; 6],
    // version: [char; 2],
    uname: String,
    gname: String,
    // devmajor: [char; 8],
    // devminor: [char; 8],
    // prefix: [char; 155],
    // pad: [char; 12]
    // mode: [char; 8],
    // uid: [char; 8],
    // gid: [char; 8],
    // size: [char; 12],
    // mtime: [char; 12],
    // checksum: [char; 8],
    // typeflag: [char; 1],
    // linkname: [char; 100],
    // magic: [char; 6],
    // version: [char; 2],
    // uname: [char; 32],
    // gname: [char; 32],
    // devmajor: [char; 8],
    // devminor: [char; 8],
    // prefix: [char; 155],
    // pad: [char; 12]
}


fn octal(n: u64, pad: usize) -> String {
    format!("{:0>pad$}", format!("{:o}", n), pad=pad)
}


// TODO: typeflag
fn generate_header(path: &std::path::Path, typeflag: [char; 1]) -> Header {
    Header {
        name: path.to_str().unwrap().to_string(),
        mode: String::from("0000777\0"),  // TODO
        uid: 1000, gid: 1000,  // TODO
        mtime: 114514,  // TODO
        typeflag: typeflag,
        checksum: 0,
        uname: "nanoha".to_string(),  // TODO
        gname: "nanoha".to_string()  // TODO
    }
}

fn create_tar_headers(org_path: &str) -> Vec<Header> {
    let path = std::path::Path::new(org_path);
    if path.is_dir() {
        let mut v = vec![generate_header(path, ['5'])];

        for p in std::fs::read_dir(path).unwrap() {
            v.append(&mut create_tar_headers(p.unwrap().path().to_str().unwrap()));
        }

        v
    } else {
        vec![generate_header(path, ['0'])]
    }
}


fn write_padding(f: &mut std::io::BufWriter<std::fs::File>, len: usize) {
    f.write_all((std::iter::repeat('\0').take(len).collect::<String>()).as_bytes()).unwrap();
}


fn write_header(f: &mut std::io::BufWriter<std::fs::File>, h: &Header) {
    let null_char = "\0".as_bytes();
    let is_file = h.typeflag[0] == '0';

    let mut header_bytes: std::vec::Vec<u8> = std::vec::Vec::new();

    // name: \0で終はる
    header_bytes.append(&mut format!("{:\0<100}", String::from(&h.name)).into_bytes());

    // mode: \0で終はる
    header_bytes.append(&mut String::from(&h.mode).into_bytes());

    // uid, gid: \0で終はる サイズ8
    header_bytes.append(&mut format!("{}\0", octal(h.uid.into(), 8 - 1)).into_bytes());
    header_bytes.append(&mut format!("{}\0", octal(h.gid.into(), 8 - 1)).into_bytes());

    // TODO: size: \0で終はる サイズ12
    // ディレクトリのsizeは0
    let size = if is_file {13} else {0};
    header_bytes.append(&mut format!("{}\0", octal(size, 12 - 1)).into_bytes());

    // mtime: \0で終はる サイズ12
    header_bytes.append(&mut format!("{}\0", octal(h.mtime.into(), 12 - 1)).into_bytes());

    // checksumは空白で埋まってゐると仮定して計算する
    // サイズ8 null+空白で終はる
    header_bytes.append(&mut vec![' ' as u8, ' ' as u8, ' ' as u8, ' ' as u8, ' ' as u8, ' ' as u8, ' ' as u8, ' ' as u8]);

    // typeflag
    header_bytes.append(&mut vec![h.typeflag[0] as u8]);

    // linkname
    header_bytes.append(&mut std::iter::repeat(0).take(100).collect::<std::vec::Vec<u8>>());

    // magic, version
    header_bytes.append(&mut vec!['u' as u8, 's' as u8, 't' as u8, 'a' as u8, 'r' as u8, ' ' as u8]);
    header_bytes.append(&mut vec![' ' as u8, '\0' as u8]);

    // uname, gname
    header_bytes.append(&mut format!("{:\0<32}", String::from(&h.uname)).into_bytes());
    header_bytes.append(&mut format!("{:\0<32}", String::from(&h.gname)).into_bytes());


    // devmajor, devminor
    header_bytes.append(&mut std::iter::repeat(0).take(8).collect::<std::vec::Vec<u8>>());
    header_bytes.append(&mut std::iter::repeat(0).take(8).collect::<std::vec::Vec<u8>>());

    // prefix
    header_bytes.append(&mut std::iter::repeat(0).take(155).collect::<std::vec::Vec<u8>>());

    // padding
    header_bytes.append(&mut std::iter::repeat(0).take(12).collect::<std::vec::Vec<u8>>());

    // calc checksum
    let mut checksum: u32 = 0;
    for &b in &header_bytes {
        checksum += b as u32;
    }

    let checksum = octal(checksum.into(), 6);
    for i in 0..6 {
        header_bytes[148+i] = checksum.chars().nth(i).unwrap() as u8;
    }

    f.write_all(&header_bytes).unwrap();
}


fn generate_tar_file(dist: &str, headers: Vec<Header>) {
    let mut f = std::io::BufWriter::new(std::fs::File::create(dist).unwrap());

    for header in headers {
        write_header(&mut f, &header);
        if header.typeflag[0] == '0' {
            let mut rf = std::fs::File::open(std::path::Path::new(&header.name)).unwrap();
            let mut buf = Vec::new();
            let _ = rf.read_to_end(&mut buf);
            f.write_all(&buf).unwrap();

            write_padding(&mut f, 512 - 13);
        }
    }

    // end of archive
    write_padding(&mut f, 1024);
}


fn main() {
    let a = create_tar_headers("test_dir");
    generate_tar_file("hoge.tar", a);
}
