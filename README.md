# TangMega-138K-AE350

## Configuring FPGA Operation in Gowin FPGA

First, it's important to note that the board has two USB Type-C ports that serve different purposes and require different drivers. For flashing the FPGA and RISC-V, you need USB-2, which is labeled as Debug.
It requires the FT2232H driver ([Download link](https://ftdichip.com/drivers/d2xx-drivers/)).

<img src="images/fig1.png" alt="Required Type-C port for flashing" width="50%" height="50%">

For successful flashing, you also need to set the switches on the board as follows:

<img src="images/fig2.png" alt="Correct switch configuration" width="50%" height="50%">

To successfully flash the board, you must also power it on. Power can be supplied directly through the USB Type-C port or through the dedicated power port.

The power port can also be seen in the figure above. When adequate power is supplied, the DC IN LED will light up.

In addition to connecting power, you need to hold the POWER button on the board for 2–5 seconds.

<img src="images/fig3.png" alt="POWER button" width="50%" height="50%">

After this, with an empty firmware on the FPGA, the POWER LED will light up.

<img src="images/fig4.png" alt="POWER, READY, DONE LEDs" width="50%" height="50%">

If this LED is not lit, the FPGA is likely not receiving power.

To write firmware for the FPGA, you need to install GOWIN EDA with a full license, otherwise access to IP cores will be limited, and working with RISC-V may be unavailable.

All resources for the FPGA and the board can be found on the [manufacturer's page](https://wiki.sipeed.com/hardware/en/tang/tang-mega-138k/mega-138k.html).

To verify FPGA operation, you can write a simple LED blinking code for the PMOD.

If the firmware was successfully synthesized and uploaded to the board, you should see the two remaining LEDs from the figure above light up, and then, if you have a PMOD-LED connected, you will see your firmware in action.

Since the main goal was to explore the operation of RISC-V within the board, let's move on to that.

## Configuring RISC-V Operation in Gowin FPGA

### Obtaining the Development Environment

To write firmware for RISC-V on the board, you need to either download the toolchain for building and compiling files from the official AndesTech GitHub, or obtain a license from Gowin for AndeSight RDS.
To get a license for AndeSight, fill out the form [at this link](https://www.gowinsemi.com/en/support/enquires/). You can fill it out following the example below.

<img src="images/fig5.png" alt="Example of fields to fill out" width="50%" height="50%">

All fields except empty ones must be filled out exactly as shown, otherwise you will either not receive a license or receive the wrong one. The license is perpetual but only allows working with one microcontroller, specifically the model that is on the same chip as the FPGA.
The response may take at least 3 days.

After receiving the key and license file, you should download AndeSight RDS from the official GOWIN website ([download link](https://cdn.gowinsemi.com.cn/RiscV_AE350_SOC_RDS_V1.3_win.zip)).

After downloading the program, on first launch a window will appear asking you to enter the license, where you enter the Serial and License File from the email.

### Preparing the Development Environment

Now it's recommended to download `demo_ae350` for building programs for this microcontroller. This build can be found in a third-party GitHub repository ([link](https://github.com/faa00/Tang_MEGA_138K_Pro_Dock)).
This repository contains 3 folders: 2 of them are projects for the FPGA, which can be ignored for now, and the `software/ae350_test/` folder is a set of files for successfully compiling test firmware for AE350 within GOWIN FPGA.
They are very useful when verifying whether you're correctly transferring the firmware file to the FPGA, and also whether you've correctly allocated the board's resources within the FPGA firmware. We'll discuss how to allocate them in the next chapter, but for now let's return to AndeSight.

First, you need to create an empty project in AndeSight, as shown below:

<img src="images/fig6.png" alt="Settings before project creation" width="50%" height="50%">
<img src="images/fig7.png" alt="Project settings" width="50%" height="50%">

For successful flashing, you need to add the `src` folder from the `ae350_test` folder to your empty project. Then right-click on the project folder and open Properties.

Now you need to configure the project build parameters. First, specify all folders containing header files and their implementations so the compiler knows where to look. In the future, if you want to create your own folders and files, you will also need to specify paths to them.

In `C/C++ Build -> Settings -> Andes C Compiler -> Directories` add the following paths:

- `${workspace_loc:/${ProjName}/src/bsp/ae350}`
- `${workspace_loc:/${ProjName}/src/bsp/config}`
- `${workspace_loc:/${ProjName}/src/bsp/driver/ae350}`
- `${workspace_loc:/${ProjName}/src/bsp/driver/include}`
- `${workspace_loc:/${ProjName}/src/bsp/lib}`
- `${workspace_loc:/${ProjName}/src/demo}`

<img src="images/fig8.png" alt="Adding paths" width="50%" height="50%">

The easiest way to do this is through the `File System` button, as you can select all paths at once, and AndeSight will substitute them automatically.

<img src="images/fig9.png" alt="File System button" width="50%" height="50%">

Now configure optimization in `C/C++ Build -> Settings -> Andes C Compiler -> Optimization`. Set the following parameters:

- Optimization Level: `-Og` (Optimize for speed with better debug ability than O1)
- Code Model: medium
- Remove unused function sections (`-ffunction-sections`): Enable
- Remove unused data sections (`-fdata-sections`): Enable

<img src="images/fig10.png" alt="Build optimization" width="50%" height="50%">

This helps conserve resources and improve debugging, although you can experiment with the settings as they barely affect the firmware.

You can also set the debug level in `C/C++ Build -> Settings -> Andes C Compiler -> Debugging`.

<img src="images/fig11.png" alt="Debugging level" width="50%" height="50%">

I set it to maximum; you can choose any level.

In `C/C++ Build -> Settings -> Andes C Compiler -> Miscellaneous` in the `Other flags` field, add: `-c -fmessage-length=0 -fno-builtin -fomit-frame-pointer -fno-strict-aliasing`, and select `gcc` as the compiler.

<img src="images/fig12.png" alt="Miscellaneous settings" width="50%" height="50%">

Besides configuring the compiler, you need to configure the linker. In `C/C++ Build -> Settings -> LdSaG Tool -> General`, set `Linker script template` to: `$(ANDESIGHT_ROOT)/utils/nds32_template_v5.txt`.
In `SaG file`, specify: `${ProjDirPath}/src/bsp/sag/ae350-ddr.sag`.

<img src="images/fig13.png" alt="LdSaG configuration" width="50%" height="50%">

Then in `C/C++ Build -> Settings -> Andes C Linker -> General`, in `Linker Script (-T)` enter: `$(LDSAG_OUT)`. The `Do not use standard start files (-nostartfiles)` option should be enabled.

<img src="images/fig14.png" alt="Linker configuration" width="50%" height="50%">

Now you can build the project by left-clicking on the project folder and selecting the hammer on the top panel.

<img src="images/fig15.png" alt="Building the project" width="50%" height="50%">

The demo project from the repository will have the `led waterfall` program and a memory test by default, where memory is allocated for arrays `a` and `b`, array `a` is filled with values, they are copied to `b`, output via UART, and then the memory is cleared and freed.

Besides these simple examples, there are others that can be enabled or disabled by changing values in `demo.h`. Based on these examples, you can write your own firmware. Unfortunately, I haven't found documentation for these tools.

All test firmware is called from `main.c`.

After building, a `debug` folder will appear containing the binary file of your firmware. It needs to be uploaded to the microcontroller, but this isn't straightforward since to flash AE350, you first need to flash the FPGA.

## Preparing the FPGA for RISC-V Operation

### Preparing the Environment

From the previously mentioned repository, you can download one of two firmware versions for the FPGA. I used `ae350_demo`, although they differ little.

Downloading them is optional, as you can configure everything yourself following the instructions below. However, I recommend taking the ready project from the repository ([link for Tang 138K](https://github.com/sipeed/TangMega-138K-example)) or ([link for Tang 138K Pro](https://github.com/sipeed/TangMega-138KPro-example/tree/main)).

If you're creating a project from scratch, use the following settings:

- Series: GW5AST
- Device: GW5AST-138
- Device Version: B
- Package: FCPBGA484A
- Speed: C1/I0
- Part Number: GW5AST-LV138FPG676AC1/I0

First, prepare the environment by connecting all IP Cores. Let's start with environment setup.

Go to `Project -> Configuration -> Global -> General` and enable DRSM to use DDR3 on the board.

<img src="images/fig16.png" alt="Enabling DRSM" width="50%" height="50%">

Next, in `Place & Route`, configure the `Place` and `Route` options as shown below:

<img src="images/fig17.png" alt="Place configuration" width="50%" height="50%">
<img src="images/fig18.png" alt="Route configuration" width="50%" height="50%">

Also redefine some pins in `Dual-Purpose Pin`. If you're not redefining JTAG, configure as shown:

<img src="images/fig19.png" alt="Pin definition" width="50%" height="50%">

### Connecting IP Cores

Add the necessary IP Cores. The first one is `RiscV AE350 SOC`, located at `Soft IP Core -> Microprocessor System -> Hard-Core-MCU`.

When adding, select what to connect to AE350. To verify functionality, it's sufficient to add GPIO and UART2, as shown:

<img src="images/fig20.png" alt="Connecting UART" width="50%" height="50%">
<img src="images/fig21.png" alt="Connecting GPIO" width="50%" height="50%">

Also add PLL for frequency conversion. At `Hard Module -> CLOCK -> PLL_ADV` add two PLLs:

1. For:
   - Clkout0: DDR clock - 50 MHz
   - Clkout1: CORE clock - 800 MHz
   - Clkout2: AHB clock - 100 MHz
   - Clkout3: APB clock - 100 MHz
   - Clkout4: RTC clock - 10 MHz

2. For DDR3:
   - Clkout0: DDR3 input clock - 50 MHz
   - Clkout2: DDR3 memory clock - 200 MHz

Configure the PLL initial page as shown:

<img src="images/fig22.png" alt="PLL configuration" width="50%" height="50%">

I also recommend adding the following module for visualizing code operation:

```verilog
// Debounce by key
module key_debounce(out, in, clk, rstn);

input  in;
input clk;      // 50MHz
output out;
input rstn;

reg in_reg0;
reg in_reg1;
reg in_reg2;

localparam st_const = 20'd1000000;  // 20ms at 50MHz

always@(posedge clk or negedge rstn)
    begin
    if (!rstn)
    begin
        in_reg0 <= 1'b0;
        in_reg1 <= 1'b0;
        in_reg2 <= 1'b0;
    end
    else
    begin
        in_reg0 <= in;
        in_reg1 <= in_reg0;
        in_reg2 <= in_reg1;
    end
end

reg [19:0] cnt;

always@(posedge clk or negedge rstn)
    begin
    if (!rstn)
    begin
        cnt <= 20'd0;
    end
    else
    begin
        if (in_reg1 == in_reg2)
        begin
            cnt <= cnt + 1'b1;
        end
        else
        begin
            cnt <= 20'd0;
        end
    end
end

reg out;

always@(posedge clk or negedge rstn)
    begin
    if (!rstn)
    begin
        out <= 1'b0;
    end
    else
    begin
        if (cnt == st_const)
        begin
            out <= in_reg2;
        end
    end
end

endmodule
```

### Top Module

Create a top module where you call all created modules and add additions for visualizing operation. Example code is in the Gowin project folder.

Now you can synthesize the firmware.

If you look carefully at the code, it might seem like we're creating a software copy of AE350, but in reality GOWIN simply connects the board's resources to the real AE350. This is evident from the resources used.
In `Utilization Summary`, you can see that few LUTs and other resources are used, and AE350 is listed as a separate item, meaning we simply enable the microcontroller through the FPGA firmware since AE350 is part of its resources.

<img src="images/fig23.png" alt="Resources used for firmware" width="50%" height="50%">

When connecting resources to the microcontroller, you can also use them through the FPGA, but do this carefully.

### Configuring Physical and Timing Constraint Files for Place&Route

For the final firmware file, you need to add `physical` and `timing constraint` files. I recommend taking them from `demo_ae350`, as they are already configured for the board. But it's important to remember that the pins in `physical constraint` are specified for Tang 138K Pro.
Therefore, they need to be adjusted using the board schematics from the official website (links above).

An example `physical constraint` is in the Gowin project folder.

Now everything is ready for flashing both the FPGA and AE350.

### Programmer Settings and Firmware Upload

You need programmer version 1.9.9 or higher, but definitely not 1.10, otherwise nothing will work.

First, flash the FPGA firmware to flash, then the AE350 firmware, which is in the `debug` folder of the AE350 project. Do as shown:

<img src="images/fig24.png" alt="Flashing AE350" width="50%" height="50%">
<img src="images/fig25.png" alt="Flashing FPGA" width="50%" height="50%">

These starting addresses are from the official GOWIN website and should work for any firmware.

To re-upload AE350 firmware, repeat the actions from the figure above.

To flash only the FPGA, repeat the actions from the figure above.

To erase flash, do as shown:

<img src="images/fig26.png" alt="Clearing flash" width="50%" height="50%">

These firmware versions don't interfere with uploading temporary firmware to SRAM for the FPGA.

If the firmware is successfully uploaded, the LED designated for DDR3 initialization will light up, and then the AE350 firmware will start.

## Possible Problems

### Board Not Appearing in Programmer

Most likely, you haven't installed the FTDI driver or you're using the wrong Type-C port. Reinstall the driver.

### Firmware Not Starting

If AE350 and DDR3 don't start after writing to flash, press the `Reconfig` button on the board several times.

### Firmware Not Starting After Power Cycling

If pressing `Reconfig` doesn't help, flash SRAM with any other firmware, then press `Reconfig`.

### Accidentally/Intentionally Redefined JTAG on the FPGA, and Now Programmer Shows "Device not found"

Erase the firmware from flash following these instructions:

1. Hold down the `Reconfig` button.
2. Start flash clearing in the programmer.
3. Wait for the `Target Device` line in the logs, then release `Reconfig`.
4. Wait for the clearing to complete.

After this, JTAG will be available again.
You can skip erasing flash and simply overwrite the FPGA firmware without redefining JTAG.
