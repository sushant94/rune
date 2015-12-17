# BV - BitVector Logic

This document records the working of BitVector Logic.

BV acts as a basic variable that the symbolic emulator uses. BitVector must be
of definite size and must emit a warning (if necessary) when an overflow
occurs.

The following operations must be supported by a structure of it to act like a
BitVector:
	* All basic operations: Add, Sub, Mul etc.
	* Array Indexing
	* Array Slicing
	* Sign Extend
	* Zero Extend

Additionally, it must also support two other operations:
	* Yeild - Write out the SMT-LIB2 Formula for the AST
	* Simplify - Recurse through the AST and simplify the formula where
	  possible.
	* Solve - This maybe implemented through a rust binding for Z3.
	  TODO: This needs more thought and discussion.

Two types, representing symbolic and concrete values respectively:
	* BitVector Symbol (BVS)
    * BitVector Value (BVV)

Interaction when an operator is applied to BVS and BVV:
	* BVS operation BVV -> BVS
	* BVS operation BVS -> BVS
	* BVV operation BVV -> BVV

Operations on Symbolic value results in building an AST incrementally.

Methods to cast from Symbolic Value to a Concrete Value and vice-versa:
	* symbolic -> concrete : Solve for the constraints. Set the value to the
	  solution. If unsat, throw error.
	* concrete -> symbolic : Make it symbolic with a single node that
	  represents the current constant value.

For convenience and to save some memory, we implement the following:
	* Implement Hash. ASTs with the same structure must hash to the same value
	* Concrete value Hash to same values anyways.
	* Be able to refer to a BV with names (variable naming).
	* Methods to pretty print the AST in human readable form.

