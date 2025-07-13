; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testFunctionWithRetrieveReturn() {
entryID0:
  %i = alloca i64, align 8
  store i64 42, ptr %i, align 4
  %vrecallID1 = load i64, ptr %i, align 4
  ret i64 %vrecallID1
}
