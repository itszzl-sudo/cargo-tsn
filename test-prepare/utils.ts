// 工具函数（没有 main）
declare function crypto_sha256(data: string): string;

export function hashData(data: string): string {
    return crypto_sha256(data);
}
