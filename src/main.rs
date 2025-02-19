//! In Module 1, we discussed Block ciphers like AES. Block ciphers have a fixed length input.
//! Real wold data that we wish to encrypt _may_ be exactly the right length, but is probably not.
//! When your data is too short, you can simply pad it up to the correct length.
//! When your data is too long, you have some options.
//!
//! In this exercise, we will explore a few of the common ways that large pieces of data can be
//! broken up and combined in order to encrypt it with a fixed-length block cipher.
//!
//! WARNING: ECB MODE IS NOT SECURE.
//! Seriously, ECB is NOT secure. Don't use it irl. We are implementing it here to understand _why_
//! it is not secure and make the point that the most straight-forward approach isn't always the
//! best, and can sometimes be trivially broken.

use std::iter::zip;

use aes::{
    cipher::{generic_array::GenericArray, BlockCipher, BlockDecrypt, BlockEncrypt, Key, KeyInit},
    Aes128,
};

///We're using AES 128 which has 16-byte (128 bit) blocks.
const BLOCK_SIZE: usize = 16;

fn main() {
    todo!("Maybe this should be a library crate. TBD");
}

/// Simple AES encryption
/// Helper function to make the core AES block cipher easier to understand.
fn aes_encrypt(data: [u8; BLOCK_SIZE], key: &[u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
    // Convert the inputs to the necessary data type
    let mut block = GenericArray::from(data);
    let key = GenericArray::from(*key);

    let cipher = Aes128::new(&key);

    cipher.encrypt_block(&mut block);

    block.into()
}

/// Simple AES encryption
/// Helper function to make the core AES block cipher easier to understand.
fn aes_decrypt(data: [u8; BLOCK_SIZE], key: &[u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
    // Convert the inputs to the necessary data type
    let mut block = GenericArray::from(data);
    let key = GenericArray::from(*key);

    let cipher = Aes128::new(&key);

    cipher.decrypt_block(&mut block);

    block.into()
}

/// Before we can begin encrypting our raw data, we need it to be a multiple of the
/// block length which is 16 bytes (128 bits) in AES128.
///
/// The padding algorithm here is actually not trivial. The trouble is that if we just
/// naively throw a bunch of zeros on the end, there is no way to know, later, whether
/// those zeros are padding, or part of the message, or some of each.
///
/// The scheme works like this. If the data is not a multiple of the block length,  we
/// compute how many pad bytes we need, and then write that number into the last several bytes.
/// Later we look at the last byte, and remove that number of bytes.
///
/// But if the data _is_ a multiple of the block length, then we have a problem. We don't want
/// to later look at the last byte and remove part of the data. Instead, in this case, we add
/// another entire block containing the block length in each byte. In our case,
/// [16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16]
fn pad(mut data: Vec<u8>) -> Vec<u8> {
    // When twe have a multiple the second term is 0
    let number_pad_bytes = BLOCK_SIZE - data.len() % BLOCK_SIZE;

    for _ in 0..number_pad_bytes {
        data.push(number_pad_bytes as u8);
    }

    data
}

/// Groups the data into BLOCK_SIZE blocks. Assumes the data is already
/// a multiple of the block size. If this is not the case, call `pad` first.
fn group(data: Vec<u8>) -> Vec<[u8; BLOCK_SIZE]> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let mut block: [u8; BLOCK_SIZE] = Default::default();
        block.copy_from_slice(&data[i..i + BLOCK_SIZE]);
        blocks.push(block);

        i += BLOCK_SIZE;
    }

    blocks
}

/// Does the opposite of the group function
fn un_group(blocks: Vec<[u8; BLOCK_SIZE]>) -> Vec<u8> {
    let mut data = Vec::new();
    for block in blocks {
        data.extend_from_slice(&block);
    }
    data
}

/// Does the opposite of the pad function.
fn un_pad(data: Vec<u8>) -> Vec<u8> {
    let last_digit = data.last().unwrap();
    let size = data.len();

    let mut is_padded = true;
    for i in size - 1..size - last_digit {
        if data[i] != last_digit {
            is_padded = false;
            break;
        }
    }
    if is_padded {
        return data.truncate(size - last_digit);
    }
    return data;
}

/// The first mode we will implement is the Electronic Code Book, or ECB mode.
/// Warning: THIS MODE IS NOT SECURE!!!!
///
/// This is probably the first thing you think of when considering how to encrypt
/// large data. In this mode we simply encrypt each block of data under the same key.
/// One good thing about this mode is that it is parallelizable. But to see why it is
/// insecure look at: https://www.ubiqsecurity.com/wp-content/uploads/2022/02/ECB2.png
fn ecb_encrypt(plain_text: Vec<u8>, key: [u8; 16]) -> Vec<u8> {
    let mut cipher_text = Vec::new();
    let padded_groups = group(pad(plain_text));
    for block in padded_groups {
        let encrypted_block = aes_encrypt(block, &key);
        cipher_text.extend_from_slice(&encrypted_block);
    }
    cipher_text
}

/// Opposite of ecb_encrypt.
fn ecb_decrypt(cipher_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    let padded_groups = un_group(un_pad(cipher_text));
    let mut plaintext = Vec::new();
    for block in padded_groups {
        let decrypted_block = aes_decrypt(block, &key);
        plaintext.extend_from_slice(&decrypted_block);
    }
    plaintext
}

fn xor(a: Vec<u8>, b: Vec<u8>) {
    zip(a, b).map(|(x, y)| x ^ y).collect::<Vec<u8>>()
}

/// The next mode, which you can implement on your own is cipherblock chaining.
/// This mode actually is secure, and it often used in real world applications.
///
/// In this mode, the ciphertext from the first block is XORed with the
/// plaintext of the next block before it is encrypted.
///
/// For more information, and a very clear diagram,
/// see https://de.wikipedia.org/wiki/Cipher_Block_Chaining_Mode
///
/// You will need to generate a random initialization vector (IV) to encrypt the
/// very first block because it doesn't have a previous block. Typically this IV
/// is inserted as the first block of ciphertext.
fn cbc_encrypt(plain_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    // Remember to generate a random initialization vector for the first block.
    let padded_groups = group(pad(plain_text));
    let cipher_text = Vec::new();
    let iv = KeyInit::new(8);
    cipher_text.append(iv);

    for i in 0..padded_groups.len() {
        let mut xored_block;
        if i == 0 {
            xored_block = xor(padded_groups[i], cipher_text[0]);
        } else {
            xored_block = xor(padded_groups[i], cipher_text[i - 1]);
        }
        let encrypted_block = aes_encrypt(xored_block, &key);
        cipher_text.extend_from_slice(&encrypted_block);
    }
    cipher_text
}

fn cbc_decrypt(cipher_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    let encrypted_groups = group(cipher_text);
    let plaintext_groups = Vec::new();
    for i in encrypted_groups.len() - 1..0 {
        let mut xored_block;
        if i == 0 {
            xored_block = xor(padded_groups[i], cipher_text[0]);
        } else {
            xored_block = xor(padded_groups[i], encrypted_groups[i - 1]);
        }
        let decrypted_block = aes_decrypt(xored_block, &key);
        plaintext_groups.push(un_pad(decrypted_block));
    }
    plaintext_groups.reverse();
    un_group(plaintext_groups)
}

/// Another mode which you can implement on your own is counter mode.
/// This mode is secure as well, and is used in real world applications.
/// It allows parallelized encryption and decryption, as well as random read access when decrypting.
///
/// In this mode, there is an index for each block being encrypted (the "counter"), as well as a random nonce.
/// For a 128-bit cipher, the nonce is 64 bits long.
///
/// For the ith block, the 128-bit value V of `nonce | counter` is constructed, where | denotes
/// concatenation. Then, V is encrypted with the key using ECB mode. Finally, the encrypted V is
/// XOR'd with the plaintext to produce the ciphertext.
///
/// A very clear diagram is present here:
/// https://en.wikipedia.org/wiki/Block_cipher_mode_of_operation#Counter_(CTR)
///
/// Once again, you will need to generate a random nonce which is 64 bits long. This should be
/// inserted as the first block of the ciphertext.
fn ctr_encrypt(plain_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    // Remember to generate a random nonce
    let nonce = [u8; BLOCK_SIZE];
    let cipher_text = Vec::new();
    let padded_groups = group(pad(plain_text));

    for i in 0..padded_groups.len() {
        nonce.append(i);
        let xored_block = xor(padded_groups[i], nonce);
        nonce.pop();
        let encrypted_block = aes_encrypt(xored_block, &key);
        cipher_text.extend_from_slice(&encrypted_block);
    }
    cipher_text
}

fn ctr_decrypt(cipher_text: Vec<u8>, key: [u8; BLOCK_SIZE]) -> Vec<u8> {
    let iv = cipher_text[0];
    let mut plaintext_groups = Vec::new();

    for i in 0..cipher_text.len() {
        iv.append(i);
        let xored_block = xor(cipher_text[i], iv);
        iv.pop();
        let decrypted_block = aes_decrypt(xored_block, &key);
        plaintext_groups.push(un_pad(decrypted_block));
    }
    un_group(plaintext_groups)
}
