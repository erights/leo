/*
namespace: Compile
expectation: Pass
input_file:
 - inputs/scalars.in
*/

function main(a: scalar, b: scalar) -> bool {
   // unary
   let i: scalar = a.square_root();

   // binary
   let j: scalar = a.add(b);
   let o: bool = a.eq(b);
   let p: bool = a.gte(b);
   let q: bool = a.gt(b);
   let r: bool = a.lte(b);
   let s: bool = a.lt(b);
   let t: group = 2group.mul(b);
   let u: group = a.mul(2group);
   let w: bool = a.neq(b);

   return a == b;
}