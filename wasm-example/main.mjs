import icu4x from "./icu4x.mjs"

function withEncodedString(str, fn) {
  let bytes = (new TextEncoder()).encode(str);

  let ptr = icu4x.icu4x_alloc(bytes.length);
  const memory = new Uint8Array(icu4x.memory.buffer);
  const buf = memory.subarray(ptr, ptr + bytes.length);
  buf.set(bytes, 0);

  try {
    return fn(ptr, buf.length);
  } finally {
    icu4x.icu4x_free(ptr, buf.length);
  }
}

function readString(ptr, len) {
  const memory = new Uint8Array(icu4x.memory.buffer);
  const buf = memory.subarray(ptr, ptr + len);
  return (new TextDecoder("utf-8")).decode(buf)
}

const fixedDecimalRegistry = new FinalizationRegistry(ptr => {
  console.log("freeing decimal!");
  icu4x.icu4x_fixed_decimal_destroy(ptr);
});

class FixedDecimal {
  constructor(magnitude) {
    this.underlying = icu4x.icu4x_fixed_decimal_create(magnitude);    
    fixedDecimalRegistry.register(this, this.underlying);
  }

  multiply_pow10(pow) {
    icu4x.icu4x_fixed_decimal_multiply_pow10(this.underlying, pow);
  }

  negate() {
    icu4x.icu4x_fixed_decimal_negate(this.underlying);
  }

  write_to(writable) {
    let outPtr = icu4x.icu4x_alloc(16);
    icu4x.icu4x_fixed_decimal_write_to(outPtr, this.underlying, writable.underlying);
    const buf = new BigUint64Array(icu4x.memory.buffer, outPtr, 2);
    const ret = {
      abc: buf[0],
      def: buf[1]
    };
    icu4x.icu4x_free(outPtr, 16);
    return ret;
  }
}

const bufferWritableRegistry = new FinalizationRegistry(ptr => {
  console.log("freeing writable!");
  icu4x.icu4x_buffer_writeable_free(ptr);
});

class BufferWritable {
  constructor() {
    this.underlying = icu4x.icu4x_buffer_writeable(0);    
    bufferWritableRegistry.register(this, this.underlying);
  }

  getString() {
    const outStringPtr = icu4x.icu4x_buffer_writeable_borrow(this.underlying);
    const outStringLen = icu4x.icu4x_buffer_writeable_len(this.underlying);
    console.log("len is", outStringLen);
    return readString(outStringPtr, outStringLen);
  }
}

const decimal = new FixedDecimal(BigInt(1234));
decimal.multiply_pow10(-2);
decimal.negate();

const outWritable = new BufferWritable();
console.log(decimal.write_to(outWritable));
console.log(outWritable.getString());
