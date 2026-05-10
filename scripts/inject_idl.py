#!/usr/bin/env python3
"""Inject sails:idl custom section into Sails WASM binary.

Usage: python3 inject_idl.py <wasm_path> <idl_path> [output_path]

For v1 Sails programs (sails-rs 0.10.x), the IDL is NOT embedded
in the WASM. This script adds a 'sails:idl' custom section so that
Gear IDE and other tooling can discover the service interface.
"""
import struct
import sys
import os


def encode_leb128(value):
    result = []
    while True:
        byte = value & 0x7F
        value >>= 7
        if value != 0:
            byte |= 0x80
        result.append(byte)
        if value == 0:
            break
    return bytes(result)


def inject_idl(wasm_path, idl_path, output_path=None):
    if output_path is None:
        base, ext = os.path.splitext(wasm_path)
        output_path = f"{base}_with_meta{ext}"

    with open(idl_path, "r") as f:
        idl_content = f.read().encode("utf-8")

    with open(wasm_path, "rb") as f:
        wasm_bytes = f.read()

    # Build custom section: section_id(0) + size(LEB128) + name_len + name + content
    name = b"sails:idl"
    section_content = bytes([len(name)]) + name + idl_content
    section_size = len(section_content)
    section = bytes([0]) + encode_leb128(section_size) + section_content

    new_wasm = wasm_bytes + section

    with open(output_path, "wb") as f:
        f.write(new_wasm)

    print(f"✅ Injected 'sails:idl' ({len(idl_content)} bytes)")
    print(f"   {os.path.basename(wasm_path)} → {os.path.basename(output_path)}")
    print(f"   WASM: {len(wasm_bytes)} → {len(new_wasm)} bytes")
    return output_path


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python3 inject_idl.py <wasm_path> <idl_path> [output_path]")
        sys.exit(1)
    inject_idl(*sys.argv[1:4])
