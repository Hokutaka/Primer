; Primer IR v0.2
; #N identifies one statement or expression in this compilation

fn %echo@0(%value@0: string) -> string {
  #0 print.string #1 %value@0:string
  #2 return #3 %value@0:string
}

#4 %left@1: string = #5 "日本語\0":string
#6 %same@2: bool = #7 eq.string(#8 call %echo@0(#9 %left@1:string):string, #10 "日本語\0":string)
#11 print.bool #12 %same@2:bool
#13 print.bool #14 and.short_circuit.bool(#15 false:bool, #16 eq.string(#17 call %echo@0(#18 "skipped":string):string, #19 %left@1:string))
