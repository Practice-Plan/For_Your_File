import os
import re
import threading
import queue
import tkinter as tk
from tkinter import ttk, filedialog, messagebox

# 图像处理库
try:
    from PIL import Image
    HAS_PIL = True
except ImportError:
    HAS_PIL = False

# 拖放支持（可选）
try:
    from tkinterdnd2 import TkinterDnD, DND_FILES
    HAS_DND = True
except ImportError:
    HAS_DND = False

class PNGResizer(TkinterDnD.Tk if HAS_DND else tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("PNG 分辨率更改器（精确尺寸）")
        self.geometry("750x550")
        self.resizable(True, True)

        # 变量
        self.file_list = []           # 待处理 PNG 路径
        self.output_dir = tk.StringVar(value="")
        self.target_size = tk.StringVar(value="32x32")   # 如 "32x32"

        self.thread_queue = queue.Queue()
        self.processing = False

        if not HAS_PIL:
            messagebox.showerror("缺少依赖", "请先安装 Pillow 库：\npip install Pillow")
            self.destroy()
            return

        self.setup_ui()
        self.after(100, self.check_queue)

    def setup_ui(self):
        # ---------- 参数设置区 ----------
        param_frame = ttk.LabelFrame(self, text="尺寸设置")
        param_frame.pack(fill=tk.X, padx=10, pady=5)

        size_frame = ttk.Frame(param_frame)
        size_frame.pack(fill=tk.X, padx=5, pady=5)
        ttk.Label(size_frame, text="目标尺寸 (宽x高)：").pack(side=tk.LEFT)
        self.size_entry = ttk.Entry(size_frame, textvariable=self.target_size, width=12)
        self.size_entry.pack(side=tk.LEFT, padx=5)
        ttk.Label(size_frame, text="例如：32x32、64x64、15x15").pack(side=tk.LEFT, padx=5)

        # 输出目录
        out_frame = ttk.Frame(param_frame)
        out_frame.pack(fill=tk.X, padx=5, pady=5)
        ttk.Label(out_frame, text="输出目录：").pack(side=tk.LEFT)
        ttk.Entry(out_frame, textvariable=self.output_dir, width=50).pack(side=tk.LEFT, padx=5, fill=tk.X, expand=True)
        ttk.Button(out_frame, text="选择", command=self.choose_output_dir).pack(side=tk.RIGHT)

        # ---------- 文件列表 ----------
        list_frame = ttk.LabelFrame(self, text="待处理 PNG 文件（可拖入）")
        list_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=5)

        self.listbox = tk.Listbox(list_frame, selectmode=tk.EXTENDED)
        self.listbox.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

        scrollbar = ttk.Scrollbar(list_frame, orient=tk.VERTICAL, command=self.listbox.yview)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        self.listbox.config(yscrollcommand=scrollbar.set)

        if HAS_DND:
            self.listbox.drop_target_register(DND_FILES)
            self.listbox.dnd_bind('<<Drop>>', self.on_drop)
            list_frame.configure(text="待处理 PNG 文件（可直接拖入）")
        else:
            list_frame.configure(text="待处理 PNG 文件（点击下方按钮添加）")

        # ---------- 按钮 ----------
        btn_frame = ttk.Frame(self)
        btn_frame.pack(fill=tk.X, padx=10, pady=5)

        ttk.Button(btn_frame, text="添加文件", command=self.add_files).pack(side=tk.LEFT, padx=5)
        ttk.Button(btn_frame, text="清空列表", command=self.clear_list).pack(side=tk.LEFT, padx=5)
        ttk.Button(btn_frame, text="移除选中", command=self.remove_selected).pack(side=tk.LEFT, padx=5)
        self.start_btn = ttk.Button(btn_frame, text="开始转换", command=self.start_convert)
        self.start_btn.pack(side=tk.RIGHT, padx=5)

        # ---------- 进度条 ----------
        self.progress = ttk.Progressbar(self, mode='determinate')
        self.progress.pack(fill=tk.X, padx=10, pady=2)

        # ---------- 日志 ----------
        log_frame = ttk.LabelFrame(self, text="日志")
        log_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=5)

        self.log_text = tk.Text(log_frame, height=8, state=tk.DISABLED)
        self.log_text.pack(fill=tk.BOTH, expand=True)

    # ---------- 输出目录 ----------
    def choose_output_dir(self):
        d = filedialog.askdirectory(title="选择输出文件夹")
        if d:
            self.output_dir.set(d)

    # ---------- 文件操作 ----------
    def add_files(self):
        files = filedialog.askopenfilenames(
            title="选择 PNG 文件",
            filetypes=[("PNG 图片", "*.png"), ("所有文件", "*.*")]
        )
        self._add_to_list(files)

    def on_drop(self, event):
        files = self.tk.splitlist(event.data)
        self._add_to_list(files)

    def _add_to_list(self, files):
        for f in files:
            f = f.strip('{}')
            if not os.path.isfile(f):
                continue
            if not f.lower().endswith('.png'):
                self.log(f"⚠ 已拒绝（非 PNG）：{os.path.basename(f)}")
                continue
            if f not in self.file_list:
                self.file_list.append(f)
                self.listbox.insert(tk.END, os.path.basename(f))

    def clear_list(self):
        self.file_list.clear()
        self.listbox.delete(0, tk.END)

    def remove_selected(self):
        selected = self.listbox.curselection()
        for i in reversed(selected):
            del self.file_list[i]
            self.listbox.delete(i)

    # ---------- 开始转换 ----------
    def start_convert(self):
        if self.processing:
            return
        if not self.file_list:
            messagebox.showwarning("无文件", "请先添加要处理的 PNG 文件")
            return

        # 解析目标尺寸
        size_text = self.target_size.get().strip()
        if not size_text:
            messagebox.showwarning("参数错误", "请输入目标尺寸")
            return

        # 匹配类似 "32x32" 的格式，允许空格，大小写 x
        match = re.fullmatch(r'\s*(\d+)\s*[xX]\s*(\d+)\s*', size_text)
        if not match:
            messagebox.showwarning("参数错误", "尺寸格式应为“宽x高”，例如 32x32")
            return
        target_w = int(match.group(1))
        target_h = int(match.group(2))
        if target_w <= 0 or target_h <= 0:
            messagebox.showwarning("参数错误", "宽度和高度必须为正整数")
            return

        # 检查输出目录
        out_dir = self.output_dir.get().strip()
        if not out_dir:
            first_dir = os.path.dirname(self.file_list[0])
            out_dir = os.path.join(first_dir, "resized")
            self.output_dir.set(out_dir)

        # 开始后台线程
        self.processing = True
        self.start_btn.configure(state=tk.DISABLED)
        self.log(f"开始转换，目标尺寸：{target_w}x{target_h}")
        threading.Thread(
            target=self.convert_worker,
            args=(target_w, target_h, out_dir),
            daemon=True
        ).start()

    def convert_worker(self, target_w, target_h, out_dir):
        os.makedirs(out_dir, exist_ok=True)
        total = len(self.file_list)
        self.thread_queue.put(("progress_max", total))
        success = 0
        failed = 0

        for idx, png_path in enumerate(self.file_list):
            if not os.path.exists(png_path):
                self.thread_queue.put(("log", f"跳过（文件不存在）：{os.path.basename(png_path)}"))
                failed += 1
                self.thread_queue.put(("progress", idx + 1))
                continue

            try:
                img = Image.open(png_path)
                orig_w, orig_h = img.size

                # 高质量缩放
                resized_img = img.resize((target_w, target_h), Image.LANCZOS)

                # 输出文件名
                base_name = os.path.splitext(os.path.basename(png_path))[0]
                out_path = os.path.join(out_dir, base_name + ".png")
                # 若输出文件已存在且不是原路径，添加序号
                if os.path.exists(out_path) and os.path.normcase(out_path) != os.path.normcase(png_path):
                    counter = 1
                    while True:
                        out_path = os.path.join(out_dir, f"{base_name}_{counter}.png")
                        if not os.path.exists(out_path):
                            break
                        counter += 1

                # 保存 PNG（无损，优化）
                resized_img.save(out_path, "PNG", optimize=True)
                self.thread_queue.put(("log", f"✓ 已转换：{os.path.basename(png_path)} → {os.path.basename(out_path)} ({orig_w}x{orig_h} → {target_w}x{target_h})"))
                success += 1
            except Exception as e:
                self.thread_queue.put(("log", f"✗ 失败：{os.path.basename(png_path)} - {str(e)}"))
                failed += 1
            self.thread_queue.put(("progress", idx + 1))

        self.thread_queue.put(("log", f"处理完成：成功 {success}，失败 {failed}"))
        self.thread_queue.put(("finish", None))

    # ---------- 队列通信 ----------
    def check_queue(self):
        try:
            while True:
                msg_type, value = self.thread_queue.get_nowait()
                if msg_type == "progress_max":
                    self.progress['maximum'] = value
                elif msg_type == "progress":
                    self.progress['value'] = value
                elif msg_type == "log":
                    self.log(value)
                elif msg_type == "finish":
                    self.processing = False
                    self.start_btn.configure(state=tk.NORMAL)
        except queue.Empty:
            pass
        self.after(100, self.check_queue)

    def log(self, message):
        self.log_text.configure(state=tk.NORMAL)
        self.log_text.insert(tk.END, message + "\n")
        self.log_text.see(tk.END)
        self.log_text.configure(state=tk.DISABLED)

if __name__ == "__main__":
    app = PNGResizer()
    app.mainloop()