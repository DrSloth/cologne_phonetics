//! Tests for generation of cologne_codes

use super::*;

#[test]
fn wikipedia() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec(b"Wikipedia", &mut outbuf);
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(
        outbuf,
        &[
            CologneCode::Class3.get(),
            CologneCode::Class4.get(),
            CologneCode::Class1.get(),
            CologneCode::Class2.get(),
        ]
    )
}

#[test]
fn mueller_luedenscheidt() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec("Müller-Lüdenscheidt".as_bytes(), &mut outbuf);
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(
        outbuf,
        &[
            CologneCode::Class6.get(),
            CologneCode::Class5.get(),
            CologneCode::Class7.get(),
            CologneCode::Space.get(),
            CologneCode::Class5.get(),
            CologneCode::Class2.get(),
            CologneCode::Class6.get(),
            CologneCode::Class8.get(),
            CologneCode::Class2.get(),
        ]
    )
}

#[test]
fn breschnew() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec("Breschnew".as_bytes(), &mut outbuf);
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(
        outbuf,
        &[
            CologneCode::Class1.get(),
            CologneCode::Class7.get(),
            CologneCode::Class8.get(),
            CologneCode::Class6.get(),
            CologneCode::Class3.get(),
        ]
    )
}

#[test]
fn veni_vidi_vici() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec("Er kam, Er sah, Er siegte".as_bytes(), &mut outbuf);
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(
        outbuf,
        &[
            // Er
            CologneCode::Class0.get(),
            CologneCode::Class7.get(),
            CologneCode::Space.get(),
            // Kam
            CologneCode::Class4.get(),
            CologneCode::Class6.get(),
            CologneCode::Space.get(),
            // Er
            CologneCode::Class0.get(),
            CologneCode::Class7.get(),
            CologneCode::Space.get(),
            // sah
            CologneCode::Class8.get(),
            CologneCode::Space.get(),
            // Er
            CologneCode::Class0.get(),
            CologneCode::Class7.get(),
            CologneCode::Space.get(),
            // siegte
            CologneCode::Class8.get(),
            CologneCode::Class4.get(),
            CologneCode::Class2.get(),
        ]
    )
}

#[test]
fn special_char_spam() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec(
        "!\"#$%&'()*+,-./0123456789:;<=>?@[\\]^_`{|}~`".as_bytes(),
        &mut outbuf,
    );
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(outbuf, &[])
}

#[test]
fn special_char_spam2() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec(
        "a!\"#$%&'()*+,-./0123456789:;<=>?@[\\]^_`{|}~a`".as_bytes(),
        &mut outbuf,
    );
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(
        outbuf,
        &[
            CologneCode::Class0.get(),
            CologneCode::Space.get(),
            CologneCode::Class0.get()
        ]
    )
}

#[test]
fn grundlagen() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec("Anhand von Grundlagen".as_bytes(), &mut outbuf);
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(
        outbuf,
        &[
            CologneCode::Class0.get(),
            CologneCode::Class6.get(),
            CologneCode::Class6.get(),
            CologneCode::Class2.get(),
            CologneCode::Space.get(),
            CologneCode::Class3.get(),
            CologneCode::Class6.get(),
            CologneCode::Space.get(),
            CologneCode::Class4.get(),
            CologneCode::Class7.get(),
            CologneCode::Class6.get(),
            CologneCode::Class2.get(),
            CologneCode::Class5.get(),
            CologneCode::Class4.get(),
            CologneCode::Class6.get(),
        ]
    )
}

#[test]
fn hacico() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec("Hacico".as_bytes(), &mut outbuf);
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(
        outbuf,
        &[
            CologneCode::Class0.get(),
            CologneCode::Class8.get(),
            CologneCode::Class4.get(),
        ]
    )
}

#[test]
fn aho_aho() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec("aho aho".as_bytes(), &mut outbuf);
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(
        outbuf,
        &[
            CologneCode::Class0.get(),
            CologneCode::Space.get(),
            CologneCode::Class0.get(),
        ]
    )
}

#[test]
fn aho_aho_aho() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec("aho aho aho".as_bytes(), &mut outbuf);
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(
        outbuf,
        &[
            CologneCode::Class0.get(),
            CologneCode::Space.get(),
            CologneCode::Class0.get(),
            CologneCode::Space.get(),
            CologneCode::Class0.get(),
        ]
    )
}

#[test]
fn aro_aro() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec("aro aro".as_bytes(), &mut outbuf);
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    assert_eq!(
        outbuf,
        &[
            CologneCode::Class0.get(),
            CologneCode::Class7.get(),
            CologneCode::Space.get(),
            CologneCode::Class0.get(),
            CologneCode::Class7.get(),
        ]
    )
}

#[test]
fn alphabet() {
    let mut outbuf = Vec::new();
    utf8_to_cologne_codes_vec(
        "A B C D E F G H I J K L M N O P Q R S T U V W X Y Z".as_bytes(),
        &mut outbuf,
    );
    let outbuf = cologne_code_vec_to_bytevec(outbuf);
    let resvec = &[
        CologneCode::Class0.get(), // .get()A
        CologneCode::Space.get(),
        CologneCode::Class1.get(), // .get()B
        CologneCode::Space.get(),
        CologneCode::Class8.get(), // .get()C
        CologneCode::Space.get(),
        CologneCode::Class2.get(), // .get()D
        CologneCode::Space.get(),
        CologneCode::Class0.get(), // .get()E
        CologneCode::Space.get(),
        CologneCode::Class3.get(), // .get()F
        CologneCode::Space.get(),
        CologneCode::Class4.get(), // .get()G
        CologneCode::Space.get(),
        // H is ignored
        CologneCode::Class0.get(), // .get()I
        CologneCode::Space.get(),
        CologneCode::Class0.get(), // .get()J
        CologneCode::Space.get(),
        CologneCode::Class4.get(), // .get()K
        CologneCode::Space.get(),
        CologneCode::Class5.get(), // .get()L
        CologneCode::Space.get(),
        CologneCode::Class6.get(), // .get()M
        CologneCode::Space.get(),
        CologneCode::Class6.get(), // .get()N
        CologneCode::Space.get(),
        CologneCode::Class0.get(), // .get()O
        CologneCode::Space.get(),
        CologneCode::Class1.get(), // .get()P
        CologneCode::Space.get(),
        CologneCode::Class4.get(), // .get()Q
        CologneCode::Space.get(),
        CologneCode::Class7.get(), // .get()R
        CologneCode::Space.get(),
        CologneCode::Class8.get(), // .get()S
        CologneCode::Space.get(),
        CologneCode::Class2.get(), // .get()T
        CologneCode::Space.get(),
        CologneCode::Class0.get(), // .get()U
        CologneCode::Space.get(),
        CologneCode::Class3.get(), // .get()V
        CologneCode::Space.get(),
        CologneCode::Class3.get(), // .get()W
        CologneCode::Space.get(),
        CologneCode::Class4.get(), // .get()X
        CologneCode::Class8.get(), // .get()X
        CologneCode::Space.get(),
        CologneCode::Class0.get(), // .get()Y
        CologneCode::Space.get(),
        CologneCode::Class8.get(), // .get()Z
    ];
    assert_eq!(outbuf, resvec);
    let mut outbuf_little = Vec::new();
    utf8_to_cologne_codes_vec(
        "a b c d e f g h i j k l m n o p q r s t u v w x y z".as_bytes(),
        &mut outbuf_little,
    );
    let outbuf_little = cologne_code_vec_to_bytevec(outbuf_little);
    assert_eq!(outbuf_little, resvec);
    assert_eq!(outbuf_little, outbuf);
}
