build_child:
	cd ./activities_project && go build -o ../bin/child_binary .

build_service:
	cargo build --release
	cp target/release/monitoring_agent ./bin/monitoring_agent

run_service:
	./bin/monitoring_agent

build_all: build_child build_service

build_and_run: build_all run_service