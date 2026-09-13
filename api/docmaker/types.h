/** @copyright 2026 Sean Kasun */
#pragma once

#include <map>
#include <vector>
#include <string>
#include <cstdint>

enum Intrinsic {
  U8, U16, U32, S8, S16, S32
};

// registers for x86 interrupts
enum Register {
  AL = 1, AH = 2, AX = 3,
  BL = 4, BH = 8, BX = 0xc,
  CL = 0x10, CH = 0x20, CX = 0x30,
  DL = 0x40, DH = 0x80, DX = 0xc0,
  BP = 0x100, SI = 0x200, DI = 0x400,
  CS = 0x800, DS = 0x1000, ES = 0x2000, SS = 0x4000,
};

enum Kind {
  isIntrinsic, isEnum, isAlias, isStruct, isUnion, isRef, isFunction, isInterrupt
};

struct Ref {
  struct Symbol *symbol = nullptr;
  int32_t pointer = 0;
  int32_t array = -1;
  std::string reg;
};

struct Field {
  std::string key;
  Kind kind;
  int32_t size = 0;
  union {
    Ref *ref;
    struct Struct *theStruct;
  };
};

struct Argument {
  std::string key;
  Ref ref;
};

struct Enum {
  Symbol *type;
  std::map<std::string, int32_t> entries;
};

struct SigKey {
  bool isWild;
  uint8_t value;
  std::string param;
  uint32_t size;
};

struct Signature {
  std::vector<SigKey> bytes;
  bool isset() const;
};

struct Function {
  std::vector<Argument> arguments;
  Ref retType;
  Signature signature;
};

struct Interrupt {
  uint8_t interrupt;
  std::map<uint16_t, uint32_t> signature;
  std::map<uint16_t, std::string> inputs;
  std::map<uint16_t, std::string> outputs;
};

struct Struct {
  std::vector<Field> fields;
};

struct Symbol {
  std::string name;
  bool isSizing = false;
  bool isSized = false;
  int32_t size = 0;
  bool isKnown = false;
  int referencedLine = 0;
  std::string referencedFile;
  int definedLine = 0;
  std::string definedFile;

  Kind kind;

  union {
    Function *asFunction;
    Struct *asStruct;
    Interrupt *asInterrupt;
    Intrinsic asIntrinsic;
    Ref *asAlias;
    Enum *asEnum;
  };
};
