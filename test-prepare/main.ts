// 测试 AST 插件检测功能

// 1. 使用官方插件 API
declare function http_get(url: string): string;
declare function fs_writeFile(path: string, content: string): void;
declare function crypto_sha256(data: string): string;

// 2. 测试函数调用（AST 检测）
function testOfficialPlugins() {
    // HTTP 插件
    let response = http_get("https://example.com");
    
    // FS 插件
    fs_writeFile("output.txt", response);
    
    // Crypto 插件
    let hash = crypto_sha256(response);
    console.log("Hash:", hash);
}

// 3. 使用自定义插件 API（官方没有）
declare function my_custom_api(param: string): string;

function testCustomPlugin() {
    let result = my_custom_api("test");
    console.log("Custom:", result);
}

// 4. 运行测试
testOfficialPlugins();
testCustomPlugin();

// 5. 测试自定义插件（官方没有）
declare function my_custom_plugin_action(data: string): string;
declare function another_custom_func(x: number): number;

function testMoreCustomPlugins() {
    let result1 = my_custom_plugin_action("test");
    let result2 = another_custom_func(42);
    console.log("Custom 1:", result1);
    console.log("Custom 2:", result2);
}
