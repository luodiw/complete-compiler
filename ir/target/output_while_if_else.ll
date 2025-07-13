; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testFunction() {
entryID0:
  br label %while_condID1

while_condID1:                                    ; preds = %mergeID2, %entryID0
  br i1 true, label %while_bodyID1, label %while_endID1

while_bodyID1:                                    ; preds = %while_condID1
  br i1 true, label %thenID2, label %elseID2

thenID2:                                          ; preds = %while_bodyID1
  ret i64 2
  br label %mergeID2

elseID2:                                          ; preds = %while_bodyID1
  ret i64 1
  br label %mergeID2

mergeID2:                                         ; preds = %elseID2, %thenID2
  br label %while_condID1

while_endID1:                                     ; preds = %while_condID1
}
