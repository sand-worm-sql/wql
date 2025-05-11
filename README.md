# WQL - **Worm Query Language**  

**A SQL-inspired query language for efficient data access and blockchain querying.**  

## 🚀 Introduction  

WQL (Worm Query Language) is designed to provide **SQL-like querying capabilities** with minimal boilerplate, making it easier for developers to interact with **structured data** efficiently. It enables seamless querying of both traditional and blockchain-based data.  

---

## 🛠 Features  

- **SQL-like syntax** → Query data with familiar SQL commands.  
- **Blockchain compatibility** → Supports querying on-chain data.  
- **Lightweight & Efficient** → Designed for speed and low overhead.  
- **Flexible Data Support** → Support Data from multiple  sources like RPC, Graph-gl, HTTP  

---

## 🚀 Getting Started  

### Installation  

> Coming soon!  

### Usage  

Example query to fetch structured data:  

```sql
SELECT * FROM sui.accounts WHERE address = "0x4d6960d097167b4e9f0512d0a04d9d2a8742b428ab6b638a40940e1b827eeb35";
```  

---

## 📖 Documentation  

> Work in progress. Stay tuned!  

---

## 📅 Roadmap  

- [ ] Expand query capabilities  
- [ ] Improve storage backend integration  
- [ ] Enhance blockchain querying features  
- [ ] Build out documentation  

---

## 🤝 Contributing  

Contributions are welcome! Feel free to open issues, suggest improvements, or submit pull requests.  

---

## 📜 License  

WQL is open-source and licensed under **Apache-2.0**.  

---

## 🙌 Acknowledgments  

WQL wouldn't be possible without the work of **GlueSQL** and **EQL**, which helped shape its architecture:  

- **[GlueSQL](https://github.com/wql/wql)** → Provided a strong foundation for SQL parsing and execution in Rust. Its embedded database model influenced WQL's structure.  
- **[EQL (EVM Query Language)](https://github.com/the-graph/eql)** → Inspired the **blockchain data querying approach**, making on-chain data retrieval more intuitive and accessible.  

While WQL is an independent project with its own goals, these technologies played a major role in shaping its core. If you're exploring **embedded SQL databases** or **on-chain queries**, we highly recommend checking them out!  
