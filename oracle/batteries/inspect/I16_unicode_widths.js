console.log(['日本語', '😀', 'é', 'a\u0301', '\u200b', 'ｆｕｌｌ'], { '日本': 1 });
console.log(Array.from({ length: 8 }, (_, i) => '日本'.repeat(i + 1)));
console.log(Array.from({ length: 8 }, (_, i) => '😀'.repeat(i + 1)));
console.log(['é'.repeat(3), 'e\u0301'.repeat(3), 'ab', 'abcdef', 'x', 'yz', 'q', 'r']);
console.table([{ name: '日本語', v: 1 }, { name: '😀', v: 22 }]);
console.log('%s|%s', '日本語', '😀'.length);
