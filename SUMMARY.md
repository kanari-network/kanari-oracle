# 🚀 Kanari Oracle - ระบบ Oracle ราคา Cryptocurrency และหุ้นแบบเรียลไทม์

## 📋 สรุปการทำงาน

ระบบ **Kanari Oracle** เป็นระบบที่พัฒนาด้วยภาษา Rust สำหรับดึงและติดตามราคาของ Cryptocurrency และหุ้นแบบเรียลไทม์ จากแหล่งข้อมูลหลายแห่ง

## ✨ คุณสมบัติหลัก

### 🔄 การดึงข้อมูลราคา
- **Cryptocurrency**: Bitcoin, Ethereum, BNB, Cardano, Solana และอื่นๆ
- **หุ้น**: Apple, Google, Microsoft, Amazon, Tesla และอื่นๆ
- **แหล่งข้อมูล**: CoinGecko, Binance, Yahoo Finance, Alpha Vantage, Finnhub

### 🛡️ ความน่าเชื่อถือ
- **Retry Mechanism**: ลองใหม่อัตโนมัติเมื่อเกิดข้อผิดพลาด
- **Fallback System**: เปลี่ยนไปใช้ API อื่นเมื่อแหล่งหลักล้มเหลว
- **Error Handling**: จัดการข้อผิดพลาดอย่างครอบคลุม

### 🎛️ การกำหนดค่า
- **API Keys**: สนับสนุน API Keys สำหรับ rate limit ที่ดีกว่า
- **Update Intervals**: กำหนดความถี่ในการอัปเดตได้
- **Symbol Selection**: เลือกสกุลเงินที่ต้องการติดตามได้

## 🚀 การใช้งาน

### คำสั่งพื้นฐาน

```bash
# ดูราคาหุ้น Apple
cargo run -- price AAPL --asset-type stock

# ดูราคา Bitcoin  
cargo run -- price BTC --asset-type crypto

# แสดงรายการสกุลเงินที่รองรับ
cargo run -- list

# เริ่มบริการ Oracle (อัปเดตทุก 30 วินาที)
cargo run -- start --interval 30
```

### ตัวอย่างผลลัพธ์

```
Current price for AAPL: $255.74
Last updated: 2025-10-01 14:38:41 UTC

Current price for TSLA: $454.52
Last updated: 2025-10-01 14:40:49 UTC
```

## 📊 แหล่งข้อมูลที่ใช้

### Cryptocurrency
- **CoinGecko**: ข้อมูลครอบคลุม, มี rate limit
- **Binance**: ข้อมูลพื้นฐาน, rate limit สูงกว่า

### หุ้น
- **Yahoo Finance**: ฟรี, ไม่ต้องใช้ API key
- **Alpha Vantage**: ต้องใช้ API key
- **Finnhub**: ต้องใช้ API key

## 🔧 โครงสร้างโปรเจค

```
src/
├── main.rs          # Entry point และ CLI
├── oracle.rs        # Core Oracle engine  
├── config.rs        # การจัดการ configuration
├── models.rs        # Data structures
├── errors.rs        # Error handling
└── fetchers/
    ├── mod.rs       # Price fetcher base
    ├── crypto.rs    # Cryptocurrency fetchers
    └── stock.rs     # Stock price fetchers
```

## 🏆 จุดเด่นของระบบ

### 1. **ความทนทาน (Resilience)**
- ระบบ retry อัตโนมัติ
- Fallback ไปยัง API อื่นเมื่อจำเป็น
- Error recovery ที่แข็งแกร่ง

### 2. **ความยืดหยุ่น (Flexibility)**  
- รองรับหลาย API sources
- กำหนดค่าได้ผ่าน JSON
- เลือกสกุลเงินที่ต้องการได้

### 3. **ประสิทธิภาพ (Performance)**
- เขียนด้วย Rust (รวดเร็ว, ปลอดภัย)
- Async/await สำหรับการทำงานพร้อมกัน
- Memory safe และ thread safe

### 4. **ง่ายต่อการใช้งาน (Usability)**
- CLI interface ที่เข้าใจง่าย  
- Documentation ครบถ้วน
- ตัวอย่างการใช้งานชัดเจน

## 📈 การทดสอบที่สำเร็จ

✅ **ดึงราคาหุ้น**: ทดสอบกับ AAPL, TSLA สำเร็จ  
✅ **แสดงรายการสัญลักษณ์**: ทำงานปกติ  
✅ **การจัดการข้อผิดพลาด**: จัดการ rate limit ได้  
✅ **CLI Interface**: ใช้งานง่าย มี help ครบถ้วน  
✅ **Configuration**: สร้าง config file อัตโนมัติ  

## 🔮 การพัฒนาต่อยอด

1. **Web Interface**: สร้าง web dashboard
2. **Database Storage**: เก็บข้อมูลย้อนหลัง
3. **Alert System**: แจ้งเตือนเมื่อราคาเปลี่ยนแปลง
4. **API Server**: เปิด REST API สำหรับ clients อื่น
5. **Chart Visualization**: แสดงกราฟราคา

## 🎯 สรุป

ระบบ **Kanari Oracle** เป็นโซลูชันที่แข็งแกร่งและน่าเชื่อถือสำหรับการติดตามราคา cryptocurrency และหุ้นแบบเรียลไทม์ ด้วยการออกแบบที่เน้นความทนทาน การจัดการข้อผิดพลาด และความยืดหยุ่นในการกำหนดค่า ทำให้สามารถใช้งานได้จริงในสภาพแวดล้อมการผลิต (production)