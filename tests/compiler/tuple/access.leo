/*
namespace: Compile
expectation: Pass
input_file:
 - inputs/bool_bool.in
*/

function main(a: bool, b: bool) -> (bool, bool) {
    let t: (bool, bool) = (a, b);
    return (t.0, t.1);
}