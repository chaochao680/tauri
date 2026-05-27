<script>
  import { TrayIcon } from '@tauri-apps/api/tray'
  import MenuBuilder from '../components/MenuBuilder.svelte'
  import { Menu } from '@tauri-apps/api/menu'

  let { onMessage } = $props()

  let icon = $state(null)
  let tooltip = $state(null)
  let title = $state(null)
  let iconAsTemplate = $state(false)
  let menuOnLeftClick = $state(true)
  let menuItems = $state([])
  let qoTitle = $state('Tauri API')
  let qoHeight = $state(300)
  let qoAbilityName = $state('TestTrayAbility')
  let qoModuleName = $state('entry')

  function onItemClick(detail) {
    onMessage(`Item ${detail.text} clicked`)
  }

  async function create() {
    TrayIcon.new({
      icon,
      tooltip,
      title,
      iconAsTemplate,
      menuOnLeftClick,
      menu: await Menu.new({
        items: menuItems.map((i) => i.item)
      }),
      quickOperation: qoAbilityName
        ? {
            title: qoTitle,
            height: qoHeight,
            abilityName: qoAbilityName,
            moduleName: qoModuleName || undefined
          }
        : undefined,
      action: (event) => onMessage(event)
    }).catch(onMessage)
  }
</script>

<div class="flex flex-col children:grow gap-2">
  <div class="flex gap-1">
    <input
      class="input grow"
      type="text"
      placeholder="Title"
      bind:value={title}
    />

    <input
      class="input grow"
      type="text"
      placeholder="Tooltip"
      bind:value={tooltip}
    />

    <label>
      <input type="checkbox" class="checkbox" bind:checked={menuOnLeftClick} />
      Menu on left click
    </label>
  </div>

  <div class="flex gap-1">
    <input
      class="input grow"
      type="text"
      placeholder="Icon path"
      bind:value={icon}
    />

    <label>
      <input type="checkbox" class="checkbox" bind:checked={iconAsTemplate} />
      Icon as template
    </label>
  </div>

  <div class="flex children:grow">
    <MenuBuilder bind:items={menuItems} itemClick={onItemClick} />
  </div>

  <div class="flex gap-1 items-center">
    <span class="font-bold text-sm">QuickOperation (OHOS):</span>
  </div>
  <div class="flex gap-1">
    <input
      class="input grow"
      type="text"
      placeholder="QO Title"
      bind:value={qoTitle}
    />
    <input
      class="input"
      type="number"
      placeholder="Height"
      bind:value={qoHeight}
      style="width: 80px"
    />
  </div>
  <div class="flex gap-1">
    <input
      class="input grow"
      type="text"
      placeholder="Ability Name"
      bind:value={qoAbilityName}
    />
    <input
      class="input grow"
      type="text"
      placeholder="Module Name"
      bind:value={qoModuleName}
    />
  </div>

  <div class="flex">
    <button class="btn" onclick={create} title="Creates the tray icon"
      >Create tray</button
    >
  </div>
</div>
