; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testFunctionWithReassign() {
entryID0:
  %test_var = alloca i64, align 8
  store i64 0, ptr %test_var, align 4
  store i64 42, ptr %test_var, align 4
}
