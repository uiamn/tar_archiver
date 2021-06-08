use std::io::Write;
use std::io::Read;
use std::os::unix::fs::MetadataExt;

struct Header {
    name: String,
    mode: String,
    uid: u32,
    gid: u32,
    mtime: u64,
    typeflag: [char; 1],
    size: u64
}


fn octal(n: u64, pad: usize) -> String {
    format!("{:0>pad$}", format!("{:o}", n), pad=pad)
}


// TODO: typeflag
fn generate_header(path: &std::path::Path, typeflag: [char; 1]) -> Header {
    let meta = std::fs::metadata(path).unwrap();

    Header {
        name: path.to_str().unwrap().to_string(),
        mode: format!("{:0>7}\0", format!("{:o}", meta.mode())),
        uid: meta.uid(),
        gid: meta.uid(),
        mtime: meta.mtime().unsigned_abs(),
        typeflag: typeflag,
        size: meta.len()
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
    let mut header_bytes: std::vec::Vec<u8> = std::vec::Vec::new();

    // name: \0で終はる
    header_bytes.append(&mut format!("{:\0<100}", String::from(&h.name)).into_bytes());

    // mode: \0で終はる
    header_bytes.append(&mut String::from(&h.mode).into_bytes());

    // uid, gid: \0で終はる サイズ8
    header_bytes.append(&mut format!("{}\0", octal(h.uid.into(), 8 - 1)).into_bytes());
    header_bytes.append(&mut format!("{}\0", octal(h.gid.into(), 8 - 1)).into_bytes());

    // size: \0で終はる サイズ12
    header_bytes.append(&mut format!("{}\0", octal(h.size, 12 - 1)).into_bytes());

    // mtime: \0で終はる サイズ12
    header_bytes.append(&mut format!("{}\0", octal(h.mtime.into(), 12 - 1)).into_bytes());

    // checksumは空白で埋まってゐると仮定して計算する
    // サイズ8 null+空白で終はる
    header_bytes.append(&mut vec![' ' as u8; 8]);

    // typeflag
    header_bytes.append(&mut vec![h.typeflag[0] as u8]);

    // linkname
    header_bytes.append(&mut vec![0; 100]);

    // magic, version
    header_bytes.append(&mut vec!['u' as u8, 's' as u8, 't' as u8, 'a' as u8, 'r' as u8, ' ' as u8]);
    header_bytes.append(&mut vec![' ' as u8, '\0' as u8]);

    // uname, gname
    header_bytes.append(&mut vec![0; 32]);
    header_bytes.append(&mut vec![0; 32]);


    // devmajor, devminor
    header_bytes.append(&mut vec![0; 8]);
    header_bytes.append(&mut vec![0; 8]);


    // prefix
    header_bytes.append(&mut vec![0; 155]);

    // padding
    header_bytes.append(&mut vec![0; 12]);

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

            write_padding(&mut f, 512usize - (&header.size % 512) as usize);
        }
    }

    // end of archive
    write_padding(&mut f, 1024);
}


fn main() {
    let a = create_tar_headers("test_dir");
    generate_tar_file("hoge.tar", a);
}
