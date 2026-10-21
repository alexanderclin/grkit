export const LANGUAGES = [
  { code: 'en', label: 'English' },
  { code: 'zh-Hans', label: '简体中文' },
  { code: 'zh-Hant', label: '繁體中文' },
  { code: 'ja', label: '日本語' },
] as const;

export type Language = (typeof LANGUAGES)[number]['code'];

/** Text with inline code: a part that starts with a backtick is code. */
export type RichText = readonly string[];

const en = {
  language: 'Language',
  /** The document title. */
  title: 'GRKit: new features for RICOH GR, no firmware flashing',
  /** The hero line under the GRKit heading. */
  lead: 'New features for RICOH GR. No firmware flashing.',
  camera: 'Camera',
  firmware: 'Firmware',
  supported: 'Supported',
  planned: 'Planned',
  experimental: 'Experimental',
  experimentalNote: (name: string) =>
    `GRKit for ${name} is experimental. It has not been tested on a camera.`,
  effects: 'Effects',
  modules: 'Modules',
  /** Separates the module names in the package summary. */
  listSeparator: ', ',
  /** Module names as the camera menu shows them in this language. */
  moduleNames: {
    imageControl: 'Image Control',
    framing: 'Framing',
    guides: 'Guides',
    dateStamp: 'Date Imprint',
    grain: 'Grain',
    diffusion: 'Diffusion',
  },
  package: 'Package',
  install: 'Install',
  previewing: 'Previewing',
  samplePhoto: 'Sample photo',
  sampleCredit: 'Photo: nagi usano · CC BY 2.0',
  useSample: 'Use the sample photo',
  decodingPhoto: 'Developing RAW photo…',
  upload: 'Open a DNG photo',
  yourPhoto: 'Your photo',
  photoError: 'This photo cannot be developed. Try another DNG.',
  previewError: 'The preview cannot be shown.',
  retryPreview: 'Retry preview',
  uploadNote: 'DNG only. Your photo stays in this browser.',
  previewNote: 'The preview is approximate. Color and exposure can differ from the camera.',
  compare: 'Compare',
  original: 'Base development',
  preview: 'Preview',
  previewOf: (name: string) => `Preview ${name}`,
  /** The text before and after the selected count. */
  count: (max: number): readonly [string, string] => ['', ` of ${max} selected`],
  selectAll: 'Select all',
  clear: 'Clear',
  include: 'Include',
  icon: 'Icon',
  effect: 'Effect',
  look: 'Target look',
  type: 'Type',
  color: 'Color',
  mono: 'Mono',
  printFilm: (code: string) => `${code} print`,
  effectCount: 'Effects',
  loading: 'Loading effects…',
  loadError: 'The effects cannot be loaded. Reload the page to try again.',
  firmwareNote: 'Before you download, check the firmware version in the camera menu.',
  download: 'Download GRKit.zip',
  building: 'Preparing GRKit.zip…',
  downloading: (received: string) => `Downloading GRKit.zip: ${received} MB…`,
  ready: 'GRKit.zip is ready.',
  buildError: 'GRKit.zip cannot be prepared. Check the connection and try again.',
  trademarks:
    'GRKit is not affiliated with or endorsed by Ricoh, Kodak, or Fujifilm. Some camera, film, and paper names are trademarks of their owners.',
  licenses: 'Effect licenses',
  thirdParty: 'Third-party licenses',
  recovery: 'Recovery guide',
  firmwareDump: 'Firmware dump guide',
  modulesNote: 'The camera menu shows only the modules that you select.',
  effectsOff: 'Image Control is off. The package has no effects.',
  effectsUnavailable: (name: string) => `GRKit has no Image Control for ${name} yet.`,
  imageControlEmpty: 'Select at least one effect. Without effects, the camera hides Image Control.',
  noModules: 'Select at least one module.',
  modulesText: {
    imageControl: 'The effects below, with live-view preview.',
    framing: 'XPan 65:24, 16:9, or 4:3 photos. The viewfinder stays 3:2 and dims the rest.',
    guides: 'Thirds, center, golden ratio, or diagonals inside the Framing area.',
    dateStamp: 'A date at the bottom right of photos, with classic, GR1s dot-matrix, or GR II digits: orange on color photos, white on black-and-white photos.',
    grain: "Film grain from the camera's grain stage, in three strengths. Photos only.",
    diffusion: 'A soft diffusion glow over the highlights, in three strengths. Photos only.', 
  },
  framingXpanOnly: 'XPan 65:24 photos. The viewfinder stays 3:2 and dims the rest.',
  steps: [
    ['Turn the camera off.'],
    ['Unzip GRKit.zip. Copy the contents of ', '`SD_ROOT', ' to the card root.'],
    [
      'Insert the card. If SD scripts are off, hold MENU while you turn the camera on, then wait until it turns itself off.',
    ],
    ['Turn the camera on. GRKit loads a few seconds after power-on. Then, in ADJ, select the sixth entry, GRK.'],
    ['Optional: remove ', '`{modeset}', ' and ', '`DEVELOP.MOD', ' from the card root.'],
  ] as readonly RichText[],
  remove: 'Remove GRKit',
  removeSteps: [
    ['Turn the camera off. Delete the ', '`SCRIPT', ' folder from the card.'],
    [
      'To turn SD scripts off too, download the files below. Copy the contents of ',
      '`SD_ROOT',
      ' to the card root.',
    ],
    ['Insert the card. Hold MENU while you turn the camera on, then wait until it turns itself off.'],
    ['Remove ', '`{modeset}', ' and ', '`DEVELOP.MOD', ' from the card root.'],
  ] as readonly RichText[],
  scriptsOff: 'Download the files that turn SD scripts off',
  scriptsOffError: 'The files cannot be prepared. Try again.',
  looks: {
    'portra-400-endura-v1': 'Kodak Portra 400 on Portra Endura paper',
    'gold-200-premier-v1': 'Kodak Gold 200 on Endura Premier paper',
    'ektar-100-edge-v1': 'Kodak Ektar 100 on Ektacolor Edge paper',
    'pro-400h-crystal-v1': 'Fujifilm Pro 400H on Crystal Archive paper',
    'xtra-400-crystal-v1': 'Fujifilm Superia X-TRA 400 on Crystal Archive paper',
    'vision3-250d-2383-v1': 'Kodak Vision3 250D 5207 on Vision 2383 print film',
    'vision3-500t-2393-v1': 'Kodak Vision3 500T 5219 on Vision Premier 2393 print film',
    'portra-800-push2-v1': 'Kodak Portra 800 pushed two stops to EI 3200, on Portra Endura paper',
    'kodachrome-64-v1': 'Kodak Kodachrome 64 slide film, scanned',
    'velvia-100-v1': 'Fujifilm Velvia 100 slide film, scanned',
    'portra-160-ultra-v1': 'Kodak Portra 160 on Ultra Endura paper',
    'trix-400-mono-v1': 'Kodak Tri-X 400 on Polymax Fine-Art paper',
    'red-filter-mono-v1': 'Kodak Tri-X 400 behind a Wratten 25 red filter',
    'fp-100c-v1': 'Fujifilm FP-100C instant film',
    'aerocolor-iv-v1': 'Kodak Aerocolor IV aerial negative on Portra Endura paper',
    'exr-200t-2383-v1': 'Kodak EXR 200T on Vision 2383 print film',
  } as Readonly<Record<string, string>>,
};

export type Strings = typeof en;

const zhHans: Strings = {
  language: '语言',
  title: 'GRKit：为 RICOH GR 加入新功能，无需刷固件',
  lead: '为 RICOH GR 加入新功能，无需刷固件。',
  camera: '相机',
  firmware: '固件',
  supported: '已支持',
  planned: '计划支持',
  experimental: '实验性',
  experimentalNote: (name) => `${name} 的 GRKit 是实验性的，未经真机测试。`,
  effects: '效果',
  modules: '模块',
  listSeparator: '、',
  /** Module names as the camera menu shows them in this language. */
  moduleNames: {
    imageControl: '影像控制',
    framing: '画幅',
    guides: '辅助线',
    dateStamp: '加印日期摄影',
    grain: '颗粒感',
    diffusion: '柔光',
  },
  package: '安装包',
  install: '安装',
  previewing: '正在预览',
  samplePhoto: '示例照片',
  sampleCredit: '照片：nagi usano · CC BY 2.0',
  useSample: '换回示例照片',
  decodingPhoto: '正在显影 RAW 照片…',
  upload: '打开 DNG 照片',
  yourPhoto: '你的照片',
  photoError: '无法显影这张照片，请换一张 DNG。',
  previewError: '无法显示预览。',
  retryPreview: '重试预览',
  uploadNote: '仅支持 DNG。照片只在本浏览器中处理。',
  previewNote: '预览为近似效果，颜色和曝光可能与机内成片不同。',
  compare: '对比',
  original: '基础显影',
  preview: '预览',
  previewOf: (name) => `预览 ${name}`,
  count: (max) => ['已选 ', ` / ${max}`],
  selectAll: '全选',
  clear: '清空',
  include: '包含',
  icon: '图标',
  effect: '效果',
  look: '模拟胶片',
  type: '类型',
  color: '彩色',
  mono: '黑白',
  printFilm: (code) => `${code} 拷贝片`,
  effectCount: '效果数量',
  loading: '正在加载效果…',
  loadError: '无法加载效果。请刷新页面后重试。',
  firmwareNote: '下载前，请在相机菜单中确认固件版本。',
  download: '下载 GRKit.zip',
  building: '正在准备 GRKit.zip…',
  downloading: (received) => `正在下载 GRKit.zip：已下载 ${received} MB…`,
  ready: 'GRKit.zip 已就绪。',
  buildError: '无法准备 GRKit.zip。请检查网络后重试。',
  trademarks:
    'GRKit 与 Ricoh、Kodak、Fujifilm 无关，也未获得它们的认可。部分相机、胶片和相纸名称是其所有者的商标。',
  licenses: '效果许可',
  thirdParty: '第三方许可',
  recovery: '恢复指南',
  firmwareDump: '固件导出指南',
  modulesNote: '相机菜单只显示你选择的模块。',
  effectsOff: '影像控制已关闭，安装包中不包含效果。',
  effectsUnavailable: (name) => `GRKit 暂不支持 ${name} 的影像控制。`,
  imageControlEmpty: '请至少选择一个效果。没有效果时，相机会隐藏影像控制。',
  noModules: '请至少选择一个模块。',
  modulesText: {
    imageControl: '使用下方选择的效果，可在实时显示中预览。',
    framing: '拍摄 XPan 65:24、16:9 或 4:3 照片。显示屏仍为 3:2，画幅外的区域变暗。',
    guides: '在画幅内显示三分法、中心、黄金分割或对角线。',
    dateStamp: '在照片右下角加印日期，字体可选经典、GR1s 点阵或 GR II：彩色照片为橙色，黑白照片为白色。',
    grain: '相机内置的颗粒感，分弱、中、强三档。仅用于照片。',
    diffusion: '为高光加上柔和的漫射辉光，分弱、中、强三档。仅用于照片。',
  },
  framingXpanOnly: '拍摄 XPan 65:24 照片。显示屏仍为 3:2，画幅外的区域变暗。',
  steps: [
    ['关闭相机。'],
    ['解压 GRKit.zip，把 ', '`SD_ROOT', ' 中的内容复制到存储卡根目录。'],
    ['插入存储卡。如未启用 SD 脚本，按住 MENU 按钮开机，等待相机自动关机。'],
    ['正常开机。GRKit 在开机几秒后才加载，之后在 ADJ 模式中选择第 6 个功能 GRK。'],
    ['可选：从存储卡根目录删除 ', '`{modeset}', ' 和 ', '`DEVELOP.MOD', '。'],
  ],
  remove: '移除 GRKit',
  removeSteps: [
    ['关闭相机。从存储卡删除 ', '`SCRIPT', ' 文件夹。'],
    ['如需同时关闭 SD 脚本，下载下面的文件，把 ', '`SD_ROOT', ' 中的内容复制到存储卡根目录。'],
    ['插入存储卡。按住 MENU 按钮开机，等待相机自动关机。'],
    ['从存储卡根目录删除 ', '`{modeset}', ' 和 ', '`DEVELOP.MOD', '。'],
  ],
  scriptsOff: '下载关闭 SD 脚本的文件',
  scriptsOffError: '无法准备文件。请重试。',
  looks: {
    'portra-400-endura-v1': 'Kodak Portra 400，Portra Endura 相纸',
    'gold-200-premier-v1': 'Kodak Gold 200，Endura Premier 相纸',
    'ektar-100-edge-v1': 'Kodak Ektar 100，Ektacolor Edge 相纸',
    'pro-400h-crystal-v1': 'Fujifilm Pro 400H，Crystal Archive 相纸',
    'xtra-400-crystal-v1': 'Fujifilm Superia X-TRA 400，Crystal Archive 相纸',
    'vision3-250d-2383-v1': 'Kodak Vision3 250D 5207，Vision 2383 拷贝片',
    'vision3-500t-2393-v1': 'Kodak Vision3 500T 5219，Vision Premier 2393 拷贝片',
    'portra-800-push2-v1': 'Kodak Portra 800 增感两档至 EI 3200，Portra Endura 相纸',
    'kodachrome-64-v1': 'Kodak Kodachrome 64 正片，直接扫描',
    'velvia-100-v1': 'Fujifilm Velvia 100 正片，直接扫描',
    'portra-160-ultra-v1': 'Kodak Portra 160，Ultra Endura 相纸',
    'trix-400-mono-v1': 'Kodak Tri-X 400，Polymax Fine-Art 相纸',
    'red-filter-mono-v1': 'Kodak Tri-X 400 加 Wratten 25 红滤镜',
    'fp-100c-v1': 'Fujifilm FP-100C 即显胶片',
    'aerocolor-iv-v1': 'Kodak Aerocolor IV 航拍负片，Portra Endura 相纸',
    'exr-200t-2383-v1': 'Kodak EXR 200T，Vision 2383 拷贝片',
  },
};

const zhHant: Strings = {
  language: '語言',
  title: 'GRKit：為 RICOH GR 加入新功能，無需刷韌體',
  lead: '為 RICOH GR 加入新功能，無需刷韌體。',
  camera: '相機',
  firmware: '韌體',
  supported: '已支援',
  planned: '規劃中',
  experimental: '實驗性',
  experimentalNote: (name) => `${name} 的 GRKit 為實驗性支援，未經實機測試。`,
  effects: '效果',
  modules: '模組',
  listSeparator: '、',
  /** Module names as the camera menu shows them in this language. */
  moduleNames: {
    imageControl: '影像控制',
    framing: '畫幅',
    guides: '輔助線',
    dateStamp: '加印日期攝像',
    grain: '粒紋感',
    diffusion: '柔光',
  },
  package: '安裝包',
  install: '安裝',
  previewing: '正在預覽',
  samplePhoto: '範例照片',
  sampleCredit: '照片：nagi usano · CC BY 2.0',
  useSample: '換回範例照片',
  decodingPhoto: '正在顯影 RAW 照片…',
  upload: '開啟 DNG 照片',
  yourPhoto: '你的照片',
  photoError: '無法顯影這張照片，請換一張 DNG。',
  previewError: '無法顯示預覽。',
  retryPreview: '重試預覽',
  uploadNote: '僅支援 DNG。照片只在此瀏覽器中處理。',
  previewNote: '預覽為近似效果，顏色與曝光可能與機內成片不同。',
  compare: '對比',
  original: '基礎顯影',
  preview: '預覽',
  previewOf: (name) => `預覽 ${name}`,
  count: (max) => ['已選 ', ` / ${max}`],
  selectAll: '全選',
  clear: '清除',
  include: '包含',
  icon: '圖示',
  effect: '效果',
  look: '模擬底片',
  type: '類型',
  color: '彩色',
  mono: '黑白',
  printFilm: (code) => `${code} 拷貝片`,
  effectCount: '效果數量',
  loading: '正在載入效果…',
  loadError: '無法載入效果。請重新整理頁面後再試。',
  firmwareNote: '下載前，請在相機功能表中確認韌體版本。',
  download: '下載 GRKit.zip',
  building: '正在準備 GRKit.zip…',
  downloading: (received) => `正在下載 GRKit.zip：已下載 ${received} MB…`,
  ready: 'GRKit.zip 已就緒。',
  buildError: '無法準備 GRKit.zip。請檢查網路後再試。',
  trademarks:
    'GRKit 與 Ricoh、Kodak、Fujifilm 無關，也未獲得其認可。部分相機、底片與相紙名稱為其所有者的商標。',
  licenses: '效果授權',
  thirdParty: '第三方授權',
  recovery: '復原指南',
  firmwareDump: '韌體匯出指南',
  modulesNote: '相機功能表只顯示你選擇的模組。',
  effectsOff: '影像控制已關閉，安裝包中不包含效果。',
  effectsUnavailable: (name) => `GRKit 暫不支援 ${name} 的影像控制。`,
  imageControlEmpty: '請至少選擇一個效果。沒有效果時，相機會隱藏影像控制。',
  noModules: '請至少選擇一個模組。',
  modulesText: {
    imageControl: '使用下方選擇的效果，可在實時顯示中預覽。',
    framing: '拍攝 XPan 65:24、16:9 或 4:3 照片。顯示屏仍為 3:2，畫幅外的區域變暗。',
    guides: '在畫幅內顯示三分法、中心、黃金分割或對角線。',
    dateStamp: '在照片右下角加印日期，字體可選經典、GR1s 點陣或 GR II：彩色照片為橙色，黑白照片為白色。',
    grain: '相機內建的粒紋感，分弱、中、強三檔。僅用於照片。',
    diffusion: '為高光加上柔和的漫射輝光，分弱、中、強三檔。僅用於照片。',
  },
  framingXpanOnly: '拍攝 XPan 65:24 照片。顯示屏仍為 3:2，畫幅外的區域變暗。',
  steps: [
    ['關閉相機。'],
    ['解壓縮 GRKit.zip，把 ', '`SD_ROOT', ' 中的內容複製到記憶卡根目錄。'],
    ['插入記憶卡。如未啟用 SD 腳本，按住 MENU 按鈕開機，等待相機自動關機。'],
    ['正常開機。GRKit 在開機幾秒後才載入，之後在 ADJ 模式中選擇第 6 個功能 GRK。'],
    ['選用：從記憶卡根目錄刪除 ', '`{modeset}', ' 和 ', '`DEVELOP.MOD', '。'],
  ],
  remove: '移除 GRKit',
  removeSteps: [
    ['關閉相機。從記憶卡刪除 ', '`SCRIPT', ' 資料夾。'],
    ['如需同時關閉 SD 腳本，下載下方的檔案，把 ', '`SD_ROOT', ' 中的內容複製到記憶卡根目錄。'],
    ['插入記憶卡。按住 MENU 按鈕開機，等待相機自動關機。'],
    ['從記憶卡根目錄刪除 ', '`{modeset}', ' 和 ', '`DEVELOP.MOD', '。'],
  ],
  scriptsOff: '下載關閉 SD 腳本的檔案',
  scriptsOffError: '無法準備檔案。請再試一次。',
  looks: {
    'portra-400-endura-v1': 'Kodak Portra 400，Portra Endura 相紙',
    'gold-200-premier-v1': 'Kodak Gold 200，Endura Premier 相紙',
    'ektar-100-edge-v1': 'Kodak Ektar 100，Ektacolor Edge 相紙',
    'pro-400h-crystal-v1': 'Fujifilm Pro 400H，Crystal Archive 相紙',
    'xtra-400-crystal-v1': 'Fujifilm Superia X-TRA 400，Crystal Archive 相紙',
    'vision3-250d-2383-v1': 'Kodak Vision3 250D 5207，Vision 2383 拷貝片',
    'vision3-500t-2393-v1': 'Kodak Vision3 500T 5219，Vision Premier 2393 拷貝片',
    'portra-800-push2-v1': 'Kodak Portra 800 增感兩檔至 EI 3200，Portra Endura 相紙',
    'kodachrome-64-v1': 'Kodak Kodachrome 64 正片，直接掃描',
    'velvia-100-v1': 'Fujifilm Velvia 100 正片，直接掃描',
    'portra-160-ultra-v1': 'Kodak Portra 160，Ultra Endura 相紙',
    'trix-400-mono-v1': 'Kodak Tri-X 400，Polymax Fine-Art 相紙',
    'red-filter-mono-v1': 'Kodak Tri-X 400 加 Wratten 25 紅濾鏡',
    'fp-100c-v1': 'Fujifilm FP-100C 即顯底片',
    'aerocolor-iv-v1': 'Kodak Aerocolor IV 航拍負片，Portra Endura 相紙',
    'exr-200t-2383-v1': 'Kodak EXR 200T，Vision 2383 拷貝片',
  },
};

const ja: Strings = {
  language: '言語',
  title: 'GRKit：RICOH GR に新しい機能を、ファームウェア書き換え不要',
  lead: 'RICOH GR に新しい機能を。ファームウェアの書き換えは不要です。',
  camera: 'カメラ',
  firmware: 'ファームウェア',
  supported: '対応',
  planned: '対応予定',
  experimental: '試験的',
  experimentalNote: (name) => `${name} 用の GRKit は試験的なものです。実機ではテストしていません。`,
  effects: 'エフェクト',
  modules: 'モジュール',
  listSeparator: '、',
  /** Module names as the camera menu shows them in this language. */
  moduleNames: {
    imageControl: 'イメージコントロール',
    framing: 'フレーミング',
    guides: 'グリッド',
    dateStamp: '日付入れ撮影',
    grain: '粒状感',
    diffusion: 'ディフュージョン',
  },
  package: 'パッケージ',
  install: 'インストール',
  previewing: 'プレビュー中',
  samplePhoto: 'サンプル写真',
  sampleCredit: '写真：nagi usano · CC BY 2.0',
  useSample: 'サンプル写真に戻す',
  decodingPhoto: 'RAW 写真を現像中…',
  upload: 'DNG 写真を開く',
  yourPhoto: '読み込んだ写真',
  photoError: 'この写真は現像できません。別の DNG をお試しください。',
  previewError: 'プレビューを表示できません。',
  retryPreview: 'プレビューを再試行',
  uploadNote: 'DNG のみ。写真はこのブラウザー内だけで処理されます。',
  previewNote: 'プレビューは目安です。色や露出はカメラで撮った写真と異なる場合があります。',
  compare: '比較',
  original: '基本現像',
  preview: 'プレビュー',
  previewOf: (name) => `${name} をプレビュー`,
  count: (max) => [`${max} 件中 `, ' 件を選択'],
  selectAll: 'すべて選択',
  clear: 'クリア',
  include: '含める',
  icon: 'アイコン',
  effect: 'エフェクト',
  look: 'フィルムルック',
  type: 'タイプ',
  color: 'カラー',
  mono: 'モノクロ',
  printFilm: (code) => `${code} プリント`,
  effectCount: 'エフェクト数',
  loading: 'エフェクトを読み込み中…',
  loadError: 'エフェクトを読み込めません。ページを再読み込みして、もう一度お試しください。',
  firmwareNote: 'ダウンロードする前に、カメラのメニューでファームウェアのバージョンを確認してください。',
  download: 'GRKit.zip をダウンロード',
  building: 'GRKit.zip を準備中…',
  downloading: (received) => `GRKit.zip をダウンロード中：${received} MB…`,
  ready: 'GRKit.zip の準備ができました。',
  buildError: 'GRKit.zip を準備できません。接続を確認して、もう一度お試しください。',
  trademarks:
    'GRKit は Ricoh、Kodak、Fujifilm とは関係がなく、これらの企業の承認も受けていません。一部のカメラ、フィルム、印画紙の名称は、各所有者の商標です。',
  licenses: 'エフェクトのライセンス',
  thirdParty: 'サードパーティーのライセンス',
  recovery: '復旧ガイド',
  firmwareDump: 'ファームウェア書き出しガイド',
  modulesNote: 'カメラのメニューには、選択したモジュールだけが表示されます。',
  effectsOff: 'イメージコントロールがオフです。パッケージにエフェクトは含まれません。',
  effectsUnavailable: (name) => `GRKit には、${name} 用のイメージコントロールはまだありません。`,
  imageControlEmpty:
    'エフェクトを 1 つ以上選択してください。エフェクトがないと、カメラはイメージコントロールを表示しません。',
  noModules: 'モジュールを 1 つ以上選択してください。',
  modulesText: {
    imageControl: '下で選んだエフェクトを使います。ライブビューでプレビューできます。',
    framing: 'XPan 65:24、16:9、4:3 の写真を撮影します。撮影画面は 3:2 のままで、範囲外が暗くなります。',
    guides: 'フレーミングの範囲内に、9 分割、画面中央、黄金比、対角線のいずれかを表示します。',
    dateStamp: '写真の右下に日付を入れます。書体はクラシック、GR1s のドットマトリクス、GR II から選べます。カラー写真ではオレンジ色、白黒写真では白になります。',
    grain: 'カメラ内蔵の粒状感を、弱・中・強の 3 段階で加えます。静止画のみ。',
    diffusion: 'ハイライトに柔らかな拡散グローを、弱・中・強の 3 段階で加えます。静止画のみ。',
  },
  framingXpanOnly: 'XPan 65:24 の写真を撮影します。撮影画面は 3:2 のままで、範囲外が暗くなります。',
  steps: [
    ['カメラの電源を切ります。'],
    ['GRKit.zip を展開し、', '`SD_ROOT', ' の中身をカードのルートにコピーします。'],
    ['カードを入れます。SD スクリプトがオフの場合は、MENU ボタンを押したまま電源を入れ、電源が自動で切れるまで待ちます。'],
    ['電源を入れます。GRKit は電源を入れてから数秒後に読み込まれます。その後、撮影モードで ADJ レバーを押し込んで ADJ モードにし、6 番目の項目「GRK」を選びます。'],
    ['任意：カードのルートから ', '`{modeset}', ' と ', '`DEVELOP.MOD', ' を削除します。'],
  ],
  remove: 'GRKit の削除',
  removeSteps: [
    ['カメラの電源を切ります。カードから ', '`SCRIPT', ' フォルダを削除します。'],
    ['SD スクリプトもオフにする場合は、下のファイルをダウンロードし、', '`SD_ROOT', ' の中身をカードのルートにコピーします。'],
    ['カードを入れます。MENU ボタンを押したまま電源を入れ、電源が自動で切れるまで待ちます。'],
    ['カードのルートから ', '`{modeset}', ' と ', '`DEVELOP.MOD', ' を削除します。'],
  ],
  scriptsOff: 'SD スクリプトをオフにするファイルをダウンロード',
  scriptsOffError: 'ファイルを準備できません。もう一度お試しください。',
  looks: {
    'portra-400-endura-v1': 'Kodak Portra 400、Portra Endura 印画紙',
    'gold-200-premier-v1': 'Kodak Gold 200、Endura Premier 印画紙',
    'ektar-100-edge-v1': 'Kodak Ektar 100、Ektacolor Edge 印画紙',
    'pro-400h-crystal-v1': 'Fujifilm Pro 400H、Crystal Archive 印画紙',
    'xtra-400-crystal-v1': 'Fujifilm Superia X-TRA 400、Crystal Archive 印画紙',
    'vision3-250d-2383-v1': 'Kodak Vision3 250D 5207、Vision 2383 プリントフィルム',
    'vision3-500t-2393-v1': 'Kodak Vision3 500T 5219、Vision Premier 2393 プリントフィルム',
    'portra-800-push2-v1': 'Kodak Portra 800 を EI 3200 まで 2 段増感、Portra Endura 印画紙',
    'kodachrome-64-v1': 'リバーサルフィルム Kodak Kodachrome 64 をスキャン',
    'velvia-100-v1': 'リバーサルフィルム Fujifilm Velvia 100 をスキャン',
    'portra-160-ultra-v1': 'Kodak Portra 160、Ultra Endura 印画紙',
    'trix-400-mono-v1': 'Kodak Tri-X 400、Polymax Fine-Art 印画紙',
    'red-filter-mono-v1': 'Wratten 25 赤フィルターを付けた Kodak Tri-X 400',
    'fp-100c-v1': 'Fujifilm FP-100C インスタントフィルム',
    'aerocolor-iv-v1': 'Kodak Aerocolor IV 航空用ネガフィルム、Portra Endura 印画紙',
    'exr-200t-2383-v1': 'Kodak EXR 200T、Vision 2383 プリントフィルム',
  },
};

export const STRINGS: Readonly<Record<Language, Strings>> = {
  en,
  'zh-Hans': zhHans,
  'zh-Hant': zhHant,
  ja,
};

const STORAGE_KEY = 'grkit.language';

function isLanguage(value: unknown): value is Language {
  return LANGUAGES.some((language) => language.code === value);
}

/** Maps browser language tags to a supported language. Traditional script wins for TW, HK, and MO. */
export function matchLanguage(tags: readonly string[]): Language {
  for (const tag of tags) {
    const lower = tag.toLowerCase();
    if (lower.startsWith('zh')) {
      return /hant|-(tw|hk|mo)\b/.test(lower) ? 'zh-Hant' : 'zh-Hans';
    }
    if (lower === 'ja' || lower.startsWith('ja-')) {
      return 'ja';
    }
    if (lower.startsWith('en')) {
      return 'en';
    }
  }
  return 'en';
}

export function initialLanguage(): Language {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (isLanguage(saved)) {
      return saved;
    }
  } catch {
    // Storage can be unavailable, for example in a private window.
  }
  return matchLanguage(navigator.languages ?? [navigator.language]);
}

/** Sets the language and the title of the document. */
export function showLanguage(language: Language): void {
  document.documentElement.lang = language;
  document.title = STRINGS[language].title;
}

/** Applies a language to the document and remembers it for the next visit. */
export function applyLanguage(language: Language): void {
  showLanguage(language);
  try {
    localStorage.setItem(STORAGE_KEY, language);
  } catch {
    // The choice then lasts only for this page view.
  }
}
