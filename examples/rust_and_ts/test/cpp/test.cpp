#include <cstdint>
#include <iostream>
#include <stdexcept>
#include <string>
#include <string_view>

#include "avro/Encoder.hh"
#include "avro/Decoder.hh"
#include "avro/Specific.hh"
#include "avro/Stream.hh"

#include "types.h"
#include "org/testorg/types.h"
#include "org/testorg/example/types.h"
#include "org/testorg/test/types.h"

static org::testorg::RecordWithTypes make_sample() {
    org::testorg::RecordWithTypes r;
    r.nullable_boolean = true;
    r.int_value = -42;
    r.long_value = 1234567890123LL;
    r.float_value = 3.5f;
    r.double_value = 2.718281828459045;
    r.bytes_value = {0xDE, 0xAD, 0xBE, 0xEF};
    r.string_value = "hello, avro!";
    r.union_value = std::string("variant-string");
    r.record_value = std::make_unique<org::testorg::Size>();
    r.record_value->length_m = 1.25f;
    r.record_value->width_m = 4.5f;
    r.enum_value = org::testorg::Suit::HEARTS;
    r.fixed_value.value = {1, 2, 3, 4, 5, 6, 7, 8};
    r.numbers = {1, 2, 3, 4, 5};
    r.ages = {{"alice", 30}, {"bob", 25}};
    r.decimal_value = {0x01, 0x02, 0x03};
    r.big_decimal_value = {0x10, 0x20};
    r.duration_value.value = {0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6};
    r.uuid_value.value = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15};
    return r;
}

#define CHECK(cond)                                                            \
    do {                                                                       \
        if (!(cond)) {                                                         \
            std::cerr << "Mismatch: " #cond " (line " << __LINE__ << ")\n";    \
            return false;                                                      \
        }                                                                      \
    } while (0)

static bool records_equal(const org::testorg::RecordWithTypes& a, const org::testorg::RecordWithTypes& b) {
    CHECK(a.nullable_boolean == b.nullable_boolean);
    CHECK(a.int_value == b.int_value);
    CHECK(a.long_value == b.long_value);
    CHECK(a.float_value == b.float_value);
    CHECK(a.double_value == b.double_value);
    CHECK(a.bytes_value == b.bytes_value);
    CHECK(a.string_value == b.string_value);
    CHECK(a.union_value == b.union_value);
    CHECK(a.record_value && b.record_value);
    CHECK(a.record_value->length_m == b.record_value->length_m);
    CHECK(a.record_value->width_m == b.record_value->width_m);
    CHECK(a.enum_value.value() == b.enum_value.value());
    CHECK(a.fixed_value.value == b.fixed_value.value);
    CHECK(a.numbers == b.numbers);
    CHECK(a.ages == b.ages);
    CHECK(a.decimal_value == b.decimal_value);
    CHECK(a.big_decimal_value == b.big_decimal_value);
    CHECK(a.duration_value.value == b.duration_value.value);
    CHECK(a.uuid_value.value == b.uuid_value.value);
    return true;
}

int main() {
    org::testorg::RecordWithTypes original = make_sample();

    auto out = avro::memoryOutputStream();
    avro::EncoderPtr encoder = avro::binaryEncoder();
    encoder->init(*out);
    avro::encode(*encoder, original);
    encoder->flush();

    auto in = avro::memoryInputStream(*out);
    avro::DecoderPtr decoder = avro::binaryDecoder();
    decoder->init(*in);
    org::testorg::RecordWithTypes decoded;
    avro::decode(*decoder, decoded);

    if (!records_equal(original, decoded)) {
        std::cerr << "RecordWithTypes round-trip FAILED\n";
        return 1;
    }
    std::cout << "RecordWithTypes round-trip OK\n";
    return 0;
}
