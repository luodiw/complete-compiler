; ModuleID = 'dummy_module'
source_filename = "dummy_module"

define i64 @testDeeplyNestedLoops() {
entryID0:
  %i = alloca i64, align 8
  store i64 5, ptr %i, align 4
  br label %for_condID1

for_condID1:                                      ; preds = %for_incID1, %entryID0
  br i1 true, label %for_bodyID1, label %for_endID1

for_bodyID1:                                      ; preds = %for_condID1
  br label %while_condID2

while_condID2:                                    ; preds = %do_endID3, %for_bodyID1
  br i1 true, label %while_bodyID2, label %while_endID2

while_bodyID2:                                    ; preds = %while_condID2
  br label %do_bodyID3

do_bodyID3:                                       ; preds = %do_condID3, %while_bodyID2
  %j = alloca i64, align 8
  store i64 6, ptr %j, align 4
  br label %for_condID4

for_condID4:                                      ; preds = %for_incID4, %do_bodyID3
  br i1 true, label %for_bodyID4, label %for_endID4

for_bodyID4:                                      ; preds = %for_condID4
  br label %while_condID5

while_condID5:                                    ; preds = %do_endID6, %for_bodyID4
  br i1 true, label %while_bodyID5, label %while_endID5

while_bodyID5:                                    ; preds = %while_condID5
  br label %do_bodyID6

do_bodyID6:                                       ; preds = %do_condID6, %while_bodyID5
  ret i64 0
  br label %do_condID6

do_condID6:                                       ; preds = %do_bodyID6
  br i1 true, label %do_bodyID6, label %do_endID6

do_endID6:                                        ; preds = %do_condID6
  br label %while_condID5

while_endID5:                                     ; preds = %while_condID5
  br label %for_incID4

for_incID4:                                       ; preds = %while_endID5
  br label %for_condID4

for_endID4:                                       ; preds = %for_condID4
  br label %do_condID3

do_condID3:                                       ; preds = %for_endID4
  br i1 true, label %do_bodyID3, label %do_endID3

do_endID3:                                        ; preds = %do_condID3
  br label %while_condID2

while_endID2:                                     ; preds = %while_condID2
  br label %for_incID1

for_incID1:                                       ; preds = %while_endID2
  br label %for_condID1

for_endID1:                                       ; preds = %for_condID1
}
