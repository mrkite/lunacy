/** @copyright 2026 Sean Kasun */
#include <iostream>
#include "parser.h"
#include "compiler.h"
#include "output.h"

int main(int argc, char **argv) {
  if (argc != 2) {
    std::cerr << "Usage: " << argv[0] << " <directory>" << std::endl;
    return -1;
  }

  Parser parser;
  parser.add("u8", U8);
  parser.add("u16", U16);
  parser.add("u32", U32);
  parser.add("i8", S8);
  parser.add("i16", S16);
  parser.add("i32", S32);
  // registers for x86
  parser.addRegister("al", AL);
  parser.addRegister("ah", AH);
  parser.addRegister("ax", AX);
  parser.addRegister("bl", BL);
  parser.addRegister("bh", BH);
  parser.addRegister("bx", BX);
  parser.addRegister("cl", CL);
  parser.addRegister("ch", CH);
  parser.addRegister("cx", CX);
  parser.addRegister("dl", DL);
  parser.addRegister("dh", DH);
  parser.addRegister("dx", DX);
  parser.addRegister("bp", BP);
  parser.addRegister("si", SI);
  parser.addRegister("di", DI);
  parser.addRegister("cs", CS);
  parser.addRegister("ds", DS);
  parser.addRegister("es", ES);
  parser.addRegister("ss", SS);

  if (!parser.run(argv[1])) {
    return -1;
  }
  Compiler compiler;
  if (!compiler.run(parser.symbols)) {
    return -1;
  }
  std::string outname;
  outname = argv[1];
  outname += ".dat";

  Output output;
  output.save(parser.symbols, outname);
  return 0;
}
