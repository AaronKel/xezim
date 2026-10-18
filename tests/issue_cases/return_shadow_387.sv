module top;
 int scratch[2]='{2,3};
 function automatic int scalar_shadow(); int scratch; scratch=17; return scratch; endfunction
 initial $display("T|shadow|%0d %0d %0d",scalar_shadow(),scratch[0],scratch[1]);
endmodule
