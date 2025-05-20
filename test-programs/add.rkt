#lang racket

(+ 1 (+ 2 (+ (+ (begin (+ 3 4) 5) (if 6 7 8)) 9)))
