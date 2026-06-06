// 第二个入口文件（也有 main 函数）
declare function os_type(): string;
declare function os_hostname(): string;

function printSystemInfo() {
    let os = os_type();
    let hostname = os_hostname();
    console.log("OS:", os);
    console.log("Hostname:", hostname);
}

// 这个文件也有 main 函数
function main() {
    console.log("=== Worker Entry Point ===");
    printSystemInfo();
}
