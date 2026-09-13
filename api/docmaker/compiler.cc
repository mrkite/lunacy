/** @copyright 2026 Sean Kasun */
#include <set>
#include <stdarg.h>
#include "compiler.h"
#include "types.h"

static bool compareSigs(const Signature &lhs, const Signature &rhs) {
  int li = 0;
  int ri = 0;
  int lWild = 0;
  int rWild = 0;
  while (li < lhs.bytes.size() && ri < rhs.bytes.size()) {
    if (lhs.bytes[li].isWild && lWild == 0) {
      lWild = lhs.bytes[li].size;
    }
    if (rhs.bytes[ri].isWild && rWild == 0) {
      rWild = rhs.bytes[ri].size;
    }
    if (lWild && rWild) {
      lWild--;
      rWild--;
    } else if (lWild) {
      ri++;
      lWild--;
    } else if (rWild) {
      li++;
      rWild--;
    } else {
      if (lhs.bytes[li++].value != rhs.bytes[ri++].value) {
        return true;
      }
    }
  }
  return false;
}

bool Compiler::run(std::map<std::string, Symbol> &symbols) {
  numErrors = 0;

  for (auto it = symbols.begin(); it != symbols.end(); it++) {
    if (!it->second.isKnown) {
      if (!refErr(&it->second, "Unknown type: '%s'\n",
                  it->second.name.c_str())) {
        return false;
      }
    } else if (!it->second.isSized) {
      calculateSizes(&it->second);
    }
  }

  std::set<Signature, bool(*)(const Signature &, const Signature &)> uniqueSigs(compareSigs);

  // verify functions only passed 32-bits or smaller
  // also verify functions have unique signatures
  for (auto it = symbols.begin(); it != symbols.end(); it++) {
    if (it->second.kind == isFunction) {
      Function *f = it->second.asFunction;
      for (auto arg = f->arguments.begin(); arg != f->arguments.end(); arg++) {
        if (arg->ref.pointer == 0 &&
            (arg->ref.symbol->size > 4 || arg->ref.array == 0)) {
          if (!defErr(&it->second, "Argument too large: '%s'\n",
                      arg->key.c_str())) {
            return false;
          }
        }
      }
      if (f->retType.symbol != nullptr && f->retType.pointer == 0 &&
          (f->retType.symbol->size > 4 || f->retType.array >= 0)) {
        if (!defErr(&it->second, "Return value too large on '%s'\n",
                    it->second.name.c_str())) {
          return false;
        }
      }
      if (f->signature.isset()) {
        for (auto sig = f->signature.bytes.begin(); sig != f->signature.bytes.end(); sig++) {
          if (sig->isWild) {
            sig->size = 0;
            for (auto arg = f->arguments.cbegin(); arg != f->arguments.cend(); arg++) {
              if (arg->key == sig->param) {
                if (arg->ref.pointer != 0) {
                  sig->size = 4;
                } else {
                  sig->size = arg->ref.symbol->size;
                }
              }
            }
            if (sig->size == 0) {
              if (!defErr(&it->second, "Signature references non-existant parameter: '%s'\n", sig->param.c_str())) {
                return false;
              }
            }
          }
        }

        if (uniqueSigs.find(f->signature) != uniqueSigs.end()) {
          if (!defErr(&it->second, "Duplicate signature on '%s'\n",
                      it->second.name.c_str())) {
            return false;
          }
        } else {
          uniqueSigs.insert(f->signature);
        }
      }
    }
  }
  return true;
}

bool Compiler::calculateSizes(Symbol *s) {
  s->isSizing = true;
  switch (s->kind) {
    case isFunction:
      s->size = 4;
      break;
    case isAlias:
    case isRef:
      s->size = calcRefSize(s->asAlias, s);
      break;
    case isStruct:
      s->size = calcStructSize(s->asStruct, s);
      if (s->size < 0) {
        return false;
      }
      break;
    case isUnion:
      s->size = calcUnionSize(s->asStruct, s);
      if (s->size < 0) {
        return false;
      }
      break;
    case isEnum:
      if (!s->asEnum->type->isKnown) {
        refErr(s->asEnum->type, "Unknown type: '%s'\n",
               s->asEnum->type->name.c_str());
        return false;
      }
      if (!s->asEnum->type->isSized) {
        if (s->asEnum->type->isSizing) {
          defErr(s, "Recurisve type found: '%s'\n", s->name.c_str());
          return false;
        }
        if (!calculateSizes(s->asEnum->type)) {
          return false;
        }
      }
      s->size = s->asEnum->type->size;
      break;
    case isInterrupt:
      s->size = 4;
      break;
    case isIntrinsic:
      switch (s->asIntrinsic) {
        case U8:
        case S8:
          s->size = 1;
          break;
        case U16:
        case S16:
          s->size = 2;
          break;
        case U32:
        case S32:
          s->size = 4;
          break;
        default:
          defErr(s, "Unknown intrinsic\n");
          return false;
      }
      break;
    default:
      defErr(s, "Unknown symbol type\n");
      return false;
  }
  s->isSized = true;
  s->isSizing = false;
  return true;
}

int32_t Compiler::calcRefSize(Ref *ref, Symbol *parent) {
  if (!ref->symbol->isKnown) {
    refErr(ref->symbol, "Unknown type: %s'\n", ref->symbol->name.c_str());
    return -1;
  }
  // prevent recursion errors
  if (ref->pointer > 0) {
    return 4;
  }
  if (!ref->symbol->isSized) {
    if (ref->symbol->isSizing) {
      defErr(parent, "Recursive type found: '%s'\n", parent->name.c_str());
      return -1;
    }
    if (!calculateSizes(ref->symbol)) {
      return -1;
    }
  }
  int32_t size = ref->symbol->size;
  if (ref->array >= 0) {
    size *= ref->array;
  }
  return size;
}

int32_t Compiler::calcStructSize(Struct *s, Symbol *parent) {
  int32_t size = 0;
  for (auto it = s->fields.begin(); it != s->fields.end(); it++) {
    switch (it->kind) {
      case isRef:
        it->size = calcRefSize(it->ref, parent);
        break;
      case isStruct:
        it->size = calcStructSize(it->theStruct, parent);
        break;
      case isUnion:
        it->size = calcUnionSize(it->theStruct, parent);
        break;
      default:
        defErr(parent, "Unknown field type: '%s'\n", it->key.c_str());
        return -1;
    }
    if (it->size < 0) {
      return -1;
    }
    size += it->size;
  }
  return size;
}

int32_t Compiler::calcUnionSize(Struct *s, Symbol *parent) {
  int32_t size = 0;
  for (auto it = s->fields.begin(); it != s->fields.end(); it++) {
    switch (it->kind) {
      case isRef:
        it->size = calcRefSize(it->ref, parent);
        break;
      case isStruct:
        it->size = calcStructSize(it->theStruct, parent);
        break;
      case isUnion:
        it->size = calcUnionSize(it->theStruct, parent);
        break;
      default:
        defErr(parent, "Unknown field type: '%s'\n", it->key.c_str());
        return -1;
    }
    if (it->size < 0) {
      return -1;
    }
    if (it->size > size) {
      size = it->size;
    }
  }
  return size;
}

bool Compiler::refErr(Symbol *symbol, const char *format, ...) {
  va_list args;
  va_start(args, format);

  fprintf(stderr, "Error: %s:%d: ", symbol->referencedFile.c_str(),
          symbol->referencedLine);
  vfprintf(stderr, format, args);

  va_end(args);

  numErrors++;
  return numErrors < 10;
}

bool Compiler::defErr(Symbol *symbol, const char *format, ...) {
  va_list args;
  va_start(args, format);

  fprintf(stderr, "Error: %s:%d: ", symbol->definedFile.c_str(),
          symbol->definedLine);
  vfprintf(stderr, format, args);

  va_end(args);

  numErrors++;
  return numErrors < 10;
}

bool Signature::isset() const {
  return !bytes.empty();
}
