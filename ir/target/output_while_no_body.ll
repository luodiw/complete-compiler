; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testFunctionWithWhileNoBody() {
entryID0:
  br label %while_condID1

while_condID1:                                    ; preds = %while_bodyID1, %entryID0
  br i1 true, label %while_bodyID1, label %while_endID1

while_bodyID1:                                    ; preds = %while_condID1
  br label %while_condID1

while_endID1:                                     ; preds = %while_condID1
}
