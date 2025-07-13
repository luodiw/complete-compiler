; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testFunctionWithDoWhileLoop() {
entryID0:
  br label %do_bodyID1

do_bodyID1:                                       ; preds = %do_condID1, %entryID0
  ret i64 24
  br label %do_condID1

do_condID1:                                       ; preds = %do_bodyID1
  br i1 true, label %do_bodyID1, label %do_endID1

do_endID1:                                        ; preds = %do_condID1
}
