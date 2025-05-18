#lang racket

(begin (begin (if (if #f 1000 2000) 3 1234) (if #t 0 1)) 999)
