/*
namespace: Compile
expectation: Pass
input_file: inputs/add.in
*/

function main(a: i16, b: i16, c: i16) -> bool {
    return a & b == c;
}
