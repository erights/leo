/*
namespace: Compile
expectation: Pass
input_file:
 - inputs/eq.in
 - inputs/ne.in
*/

function main(a: u64, b: u64, c: bool) -> bool{
    return (a != b) == c;
}
