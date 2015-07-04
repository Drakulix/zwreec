ee.rs: ./zop.py nyan.zop end.zop pre.zop convert.py
	./zop.py > func.zop
	cat nyan.zop func.zop | ./convert.py > conv.zop
	cat pre.zop conv.zop end.zop > ee.rs
	rm conv.zop func.zop
	echo copy ee.rs to ../src/zwreec/backend/zcode/ee.rs yourself
