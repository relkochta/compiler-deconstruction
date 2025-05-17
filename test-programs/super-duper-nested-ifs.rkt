#lang racket

(if (if #f 1000 2000) 3 (if #f (if #t (if #t #f #t) 200) 17))
